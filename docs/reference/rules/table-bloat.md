# tableBloat

**Diagnostic Category: `splinter/performance/tableBloat`**

**Severity**: Info

## Description

Detects if a table has excess bloat and may benefit from maintenance operations like vacuum full or cluster.

## Remediation

Consider running vacuum full (WARNING: incurs downtime) and tweaking autovacuum settings to reduce bloat.

## SQL Query

```sql
(
with constants as (
    select current_setting('block_size')::numeric as bs, 23 as hdr, 4 as ma
),

bloat_info as (
    select
        ma,
        bs,
        schemaname,
        tablename,
        (datawidth + (hdr + ma - (case when hdr % ma = 0 then ma else hdr % ma end)))::numeric as datahdr,
        (maxfracsum * (nullhdr + ma - (case when nullhdr % ma = 0 then ma else nullhdr % ma end))) as nullhdr2
    from (
        select
            schemaname,
            tablename,
            hdr,
            ma,
            bs,
            sum((1 - null_frac) * avg_width) as datawidth,
            max(null_frac) as maxfracsum,
            hdr + (
                select 1 + count(*) / 8
                from pg_stats s2
                where
                    null_frac <> 0
                    and s2.schemaname = s.schemaname
                    and s2.tablename = s.tablename
            ) as nullhdr
        from pg_stats s, constants
        group by 1, 2, 3, 4, 5
    ) as foo
),

table_bloat as (
    select
        schemaname,
        tablename,
        cc.relpages,
        bs,
        ceil((cc.reltuples * ((datahdr + ma -
          (case when datahdr % ma = 0 then ma else datahdr % ma end)) + nullhdr2 + 4)) / (bs - 20::float)) as otta
    from
        bloat_info
        join pg_class cc
            on cc.relname = bloat_info.tablename
        join pg_namespace nn
            on cc.relnamespace = nn.oid
            and nn.nspname = bloat_info.schemaname
            and nn.nspname <> 'information_schema'
        where
            cc.relkind = 'r'
            and cc.relam = (select oid from pg_am where amname = 'heap')
),

bloat_data as (
    select
        'table' as type,
        schemaname,
        tablename as object_name,
        round(case when otta = 0 then 0.0 else table_bloat.relpages / otta::numeric end, 1) as bloat,
        case when relpages < otta then 0 else (bs * (table_bloat.relpages - otta)::bigint)::bigint end as raw_waste
    from
        table_bloat
)

select
    'table_bloat' as "name!",
    'Table Bloat' as "title!",
    'INFO' as "level!",
    'EXTERNAL' as "facing!",
    array['PERFORMANCE'] as "categories!",
    'Detects if a table has excess bloat and may benefit from maintenance operations like vacuum full or cluster.' as "description!",
    format(
        'Table `%s`.`%s` has excessive bloat',
        bloat_data.schemaname,
        bloat_data.object_name
    ) as "detail!",
    'Consider running vacuum full (WARNING: incurs downtime) and tweaking autovacuum settings to reduce bloat.' as "remediation!",
    jsonb_build_object(
        'schema', bloat_data.schemaname,
        'name', bloat_data.object_name,
        'type', bloat_data.type
    ) as "metadata!",
    format(
        'table_bloat_%s_%s',
        bloat_data.schemaname,
        bloat_data.object_name
    ) as "cache_key!"
from
    bloat_data
where
    bloat > 70.0
    and raw_waste > (20 * 1024 * 1024) -- filter for waste > 200 MB
order by
    schemaname,
    object_name)
```

## How to configure

```json
{
  "splinter": {
    "rules": {
      "performance": {
        "tableBloat": "error"
      }
    }
  }
}
```
