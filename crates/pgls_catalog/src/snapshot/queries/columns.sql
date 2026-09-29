with
  available_tables as (
    select
      c.relname as table_name,
      c.oid as table_oid,
      c.relkind as class_kind,
      n.nspname as schema_name
    from
      pg_catalog.pg_class c
      join pg_catalog.pg_namespace n on n.oid = c.relnamespace
    where
      -- r: normal tables
      -- v: views
      -- m: materialized views
      -- f: foreign tables
      -- p: partitioned tables
      c.relkind in ('r', 'v', 'm', 'f', 'p')
  ),
  available_indexes as (
    select
      unnest (ix.indkey) as attnum,
      bool_or(ix.indisprimary) as is_primary,
      bool_or(ix.indisunique) as is_unique,
      ix.indrelid as table_oid
    from
      pg_catalog.pg_class c
      join pg_catalog.pg_index ix on c.oid = ix.indexrelid
    where
      c.relkind = 'i'
    group by table_oid, attnum
  )
select
  atts.attname as name,
  atts.attnum as number,
  ts.table_name,
  ts.table_oid :: int8 as "table_oid!",
  ts.class_kind :: char as "class_kind!",
  ts.schema_name,
  atts.atttypid :: int8 as "type_id!",
  tps.typname as "type_name",
  not atts.attnotnull as "is_nullable!",
  nullif(
    information_schema._pg_char_max_length (atts.atttypid, atts.atttypmod),
    -1
  ) as varchar_length,
  pg_get_expr (def.adbin, def.adrelid) as default_expr,
  coalesce(ix.is_primary, false) as "is_primary_key!",
  coalesce(ix.is_unique, false) as "is_unique!",
  pg_catalog.col_description (ts.table_oid, atts.attnum) as comment
from
  pg_catalog.pg_attribute atts
  join available_tables ts on atts.attrelid = ts.table_oid
  left join available_indexes ix on atts.attrelid = ix.table_oid
  and atts.attnum = ix.attnum
  left join pg_catalog.pg_attrdef def on atts.attrelid = def.adrelid
  and atts.attnum = def.adnum
  left join pg_catalog.pg_type tps on atts.atttypid = tps.oid
where
  -- system columns, such as `cmax` or `tableoid`, have negative `attnum`s
  atts.attnum >= 0 and atts.atttypid is not null and tps.oid is not null
order by
  schema_name desc,
  table_name,
  atts.attnum;