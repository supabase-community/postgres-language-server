insert into app.package_versions(package_id, version_struct, sql, description_md)
values (
(select id from app.packages where package_alias = 'michelp@adminpack'),
(0,0,2),
$adminpack$
-- From: https://github.com/ioguix/pgsql-bloat-estimation

-- Copyright (c) 2015-2019, Jehan-Guillaume (ioguix) de Rorthais
-- All rights reserved.

-- Redistribution and use in source and binary forms, with or without
-- modification, are permitted provided that the following conditions are met:

-- * Redistributions of source code must retain the above copyright notice, this
--   list of conditions and the following disclaimer.

-- * Redistributions in binary form must reproduce the above copyright notice,
--   this list of conditions and the following disclaimer in the documentation
--   and/or other materials provided with the distribution.

-- THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
-- AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
-- IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
-- DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
-- FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
-- DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
-- SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
-- CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
-- OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
-- OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

-- WARNING: executed with a non-superuser role, the query inspect only index on tables you are granted to read.
-- WARNING: rows with is_na = 't' are known to have bad statistics ("name" type is not supported).
-- This query is compatible with PostgreSQL 8.2 and after
CREATE VIEW index_bloat AS SELECT current_database(), nspname AS schemaname, tblname, idxname, bs*(relpages)::bigint AS real_size,
  bs*(relpages-est_pages)::bigint AS extra_size,
  100 * (relpages-est_pages)::float / relpages AS extra_pct,
  fillfactor,
  CASE WHEN relpages > est_pages_ff
    THEN bs*(relpages-est_pages_ff)
    ELSE 0
  END AS bloat_size,
  100 * (relpages-est_pages_ff)::float / relpages AS bloat_pct,
  is_na
  -- , 100-(pst).avg_leaf_density AS pst_avg_bloat, est_pages, index_tuple_hdr_bm, maxalign, pagehdr, nulldatawidth, nulldatahdrwidth, reltuples, relpages -- (DEBUG INFO)
FROM (
  SELECT coalesce(1 +
         ceil(reltuples/floor((bs-pageopqdata-pagehdr)/(4+nulldatahdrwidth)::float)), 0 -- ItemIdData size + computed avg size of a tuple (nulldatahdrwidth)
      ) AS est_pages,
      coalesce(1 +
         ceil(reltuples/floor((bs-pageopqdata-pagehdr)*fillfactor/(100*(4+nulldatahdrwidth)::float))), 0
      ) AS est_pages_ff,
      bs, nspname, tblname, idxname, relpages, fillfactor, is_na
      -- , pgstatindex(idxoid) AS pst, index_tuple_hdr_bm, maxalign, pagehdr, nulldatawidth, nulldatahdrwidth, reltuples -- (DEBUG INFO)
  FROM (
      SELECT maxalign, bs, nspname, tblname, idxname, reltuples, relpages, idxoid, fillfactor,
            ( index_tuple_hdr_bm +
                maxalign - CASE -- Add padding to the index tuple header to align on MAXALIGN
                  WHEN index_tuple_hdr_bm%maxalign = 0 THEN maxalign
                  ELSE index_tuple_hdr_bm%maxalign
                END
              + nulldatawidth + maxalign - CASE -- Add padding to the data to align on MAXALIGN
                  WHEN nulldatawidth = 0 THEN 0
                  WHEN nulldatawidth::integer%maxalign = 0 THEN maxalign
                  ELSE nulldatawidth::integer%maxalign
                END
            )::numeric AS nulldatahdrwidth, pagehdr, pageopqdata, is_na
            -- , index_tuple_hdr_bm, nulldatawidth -- (DEBUG INFO)
      FROM (
          SELECT n.nspname, i.tblname, i.idxname, i.reltuples, i.relpages,
              i.idxoid, i.fillfactor, current_setting('block_size')::numeric AS bs,
              CASE -- MAXALIGN: 4 on 32bits, 8 on 64bits (and mingw32 ?)
                WHEN version() ~ 'mingw32' OR version() ~ '64-bit|x86_64|ppc64|ia64|amd64' THEN 8
                ELSE 4
              END AS maxalign,
              /* per page header, fixed size: 20 for 7.X, 24 for others */
              24 AS pagehdr,
              /* per page btree opaque data */
              16 AS pageopqdata,
              /* per tuple header: add IndexAttributeBitMapData if some cols are null-able */
              CASE WHEN max(coalesce(s.null_frac,0)) = 0
                  THEN 8 -- IndexTupleData size
                  ELSE 8 + (( 32 + 8 - 1 ) / 8) -- IndexTupleData size + IndexAttributeBitMapData size ( max num filed per index + 8 - 1 /8)
              END AS index_tuple_hdr_bm,
              /* data len: we remove null values save space using it fractionnal part from stats */
              sum( (1-coalesce(s.null_frac, 0)) * coalesce(s.avg_width, 1024)) AS nulldatawidth,
              max( CASE WHEN i.atttypid = 'pg_catalog.name'::regtype THEN 1 ELSE 0 END ) > 0 AS is_na
          FROM (
              SELECT ct.relname AS tblname, ct.relnamespace, ic.idxname, ic.attpos, ic.indkey, ic.indkey[ic.attpos], ic.reltuples, ic.relpages, ic.tbloid, ic.idxoid, ic.fillfactor,
                  coalesce(a1.attnum, a2.attnum) AS attnum, coalesce(a1.attname, a2.attname) AS attname, coalesce(a1.atttypid, a2.atttypid) AS atttypid,
                  CASE WHEN a1.attnum IS NULL
                  THEN ic.idxname
                  ELSE ct.relname
                  END AS attrelname
              FROM (
                  SELECT idxname, reltuples, relpages, tbloid, idxoid, fillfactor, indkey,
                      pg_catalog.generate_series(1,indnatts) AS attpos
                  FROM (
                      SELECT ci.relname AS idxname, ci.reltuples, ci.relpages, i.indrelid AS tbloid,
                          i.indexrelid AS idxoid,
                          coalesce(substring(
                              array_to_string(ci.reloptions, ' ')
                              from 'fillfactor=([0-9]+)')::smallint, 90) AS fillfactor,
                          i.indnatts,
                          pg_catalog.string_to_array(pg_catalog.textin(
                              pg_catalog.int2vectorout(i.indkey)),' ')::int[] AS indkey
                      FROM pg_catalog.pg_index i
                      JOIN pg_catalog.pg_class ci ON ci.oid = i.indexrelid
                      WHERE ci.relam=(SELECT oid FROM pg_am WHERE amname = 'btree')
                      AND ci.relpages > 0
                  ) AS idx_data
              ) AS ic
              JOIN pg_catalog.pg_class ct ON ct.oid = ic.tbloid
              LEFT JOIN pg_catalog.pg_attribute a1 ON
                  ic.indkey[ic.attpos] <> 0
                  AND a1.attrelid = ic.tbloid
                  AND a1.attnum = ic.indkey[ic.attpos]
              LEFT JOIN pg_catalog.pg_attribute a2 ON
                  ic.indkey[ic.attpos] = 0
                  AND a2.attrelid = ic.idxoid
                  AND a2.attnum = ic.attpos
            ) i
            JOIN pg_catalog.pg_namespace n ON n.oid = i.relnamespace
            JOIN pg_catalog.pg_stats s ON s.schemaname = n.nspname
                                      AND s.tablename = i.attrelname
                                      AND s.attname = i.attname
            GROUP BY 1,2,3,4,5,6,7,8,9,10,11
      ) AS rows_data_stats
  ) AS rows_hdr_pdg_stats
) AS relation_stats
ORDER BY nspname, tblname, idxname;

CREATE VIEW table_bloat AS /* WARNING: executed with a non-superuser role, the query inspect only tables and materialized view (9.3+) you are granted to read.
* This query is compatible with PostgreSQL 9.0 and more
*/
SELECT current_database(), schemaname, tblname, bs*tblpages AS real_size,
  (tblpages-est_tblpages)*bs AS extra_size,
  CASE WHEN tblpages > 0 AND tblpages - est_tblpages > 0
    THEN 100 * (tblpages - est_tblpages)/tblpages::float
    ELSE 0
  END AS extra_pct, fillfactor,
  CASE WHEN tblpages - est_tblpages_ff > 0
    THEN (tblpages-est_tblpages_ff)*bs
    ELSE 0
  END AS bloat_size,
  CASE WHEN tblpages > 0 AND tblpages - est_tblpages_ff > 0
    THEN 100 * (tblpages - est_tblpages_ff)/tblpages::float
    ELSE 0
  END AS bloat_pct, is_na
  -- , tpl_hdr_size, tpl_data_size, (pst).free_percent + (pst).dead_tuple_percent AS real_frag -- (DEBUG INFO)
FROM (
  SELECT ceil( reltuples / ( (bs-page_hdr)/tpl_size ) ) + ceil( toasttuples / 4 ) AS est_tblpages,
    ceil( reltuples / ( (bs-page_hdr)*fillfactor/(tpl_size*100) ) ) + ceil( toasttuples / 4 ) AS est_tblpages_ff,
    tblpages, fillfactor, bs, tblid, schemaname, tblname, heappages, toastpages, is_na
    -- , tpl_hdr_size, tpl_data_size, pgstattuple(tblid) AS pst -- (DEBUG INFO)
  FROM (
    SELECT
      ( 4 + tpl_hdr_size + tpl_data_size + (2*ma)
        - CASE WHEN tpl_hdr_size%ma = 0 THEN ma ELSE tpl_hdr_size%ma END
        - CASE WHEN ceil(tpl_data_size)::int%ma = 0 THEN ma ELSE ceil(tpl_data_size)::int%ma END
      ) AS tpl_size, bs - page_hdr AS size_per_block, (heappages + toastpages) AS tblpages, heappages,
      toastpages, reltuples, toasttuples, bs, page_hdr, tblid, schemaname, tblname, fillfactor, is_na
      -- , tpl_hdr_size, tpl_data_size
    FROM (
      SELECT
        tbl.oid AS tblid, ns.nspname AS schemaname, tbl.relname AS tblname, tbl.reltuples,
        tbl.relpages AS heappages, coalesce(toast.relpages, 0) AS toastpages,
        coalesce(toast.reltuples, 0) AS toasttuples,
        coalesce(substring(
          array_to_string(tbl.reloptions, ' ')
          FROM 'fillfactor=([0-9]+)')::smallint, 100) AS fillfactor,
        current_setting('block_size')::numeric AS bs,
        CASE WHEN version()~'mingw32' OR version()~'64-bit|x86_64|ppc64|ia64|amd64' THEN 8 ELSE 4 END AS ma,
        24 AS page_hdr,
        23 + CASE WHEN MAX(coalesce(s.null_frac,0)) > 0 THEN ( 7 + count(s.attname) ) / 8 ELSE 0::int END
           + CASE WHEN bool_or(att.attname = 'oid' and att.attnum < 0) THEN 4 ELSE 0 END AS tpl_hdr_size,
        sum( (1-coalesce(s.null_frac, 0)) * coalesce(s.avg_width, 0) ) AS tpl_data_size,
        bool_or(att.atttypid = 'pg_catalog.name'::regtype)
          OR sum(CASE WHEN att.attnum > 0 THEN 1 ELSE 0 END) <> count(s.attname) AS is_na
      FROM pg_attribute AS att
        JOIN pg_class AS tbl ON att.attrelid = tbl.oid
        JOIN pg_namespace AS ns ON ns.oid = tbl.relnamespace
        LEFT JOIN pg_stats AS s ON s.schemaname=ns.nspname
          AND s.tablename = tbl.relname AND s.inherited=false AND s.attname=att.attname
        LEFT JOIN pg_class AS toast ON tbl.reltoastrelid = toast.oid
      WHERE NOT att.attisdropped
        AND tbl.relkind in ('r','m')
      GROUP BY 1,2,3,4,5,6,7,8,9,10
      ORDER BY 2,3
    ) AS s
  ) AS s2
) AS s3
-- WHERE NOT is_na
--   AND tblpages*((pst).free_percent + (pst).dead_tuple_percent)::float4/100 >= 1
ORDER BY schemaname, tblname;

-- From https://wiki.postgresql.org/wiki/Lock_dependency_information

CREATE OR REPLACE VIEW blocking_pid_tree AS
WITH RECURSIVE
  lock_composite(requested, current) AS (VALUES
    ('AccessShareLock'::text, 'AccessExclusiveLock'::text),
    ('RowShareLock'::text, 'ExclusiveLock'::text),
    ('RowShareLock'::text, 'AccessExclusiveLock'::text),
    ('RowExclusiveLock'::text, 'ShareLock'::text),
    ('RowExclusiveLock'::text, 'ShareRowExclusiveLock'::text),
    ('RowExclusiveLock'::text, 'ExclusiveLock'::text),
    ('RowExclusiveLock'::text, 'AccessExclusiveLock'::text),
    ('ShareUpdateExclusiveLock'::text, 'ShareUpdateExclusiveLock'::text),
    ('ShareUpdateExclusiveLock'::text, 'ShareLock'::text),
    ('ShareUpdateExclusiveLock'::text, 'ShareRowExclusiveLock'::text),
    ('ShareUpdateExclusiveLock'::text, 'ExclusiveLock'::text),
    ('ShareUpdateExclusiveLock'::text, 'AccessExclusiveLock'::text),
    ('ShareLock'::text, 'RowExclusiveLock'::text),
    ('ShareLock'::text, 'ShareUpdateExclusiveLock'::text),
    ('ShareLock'::text, 'ShareRowExclusiveLock'::text),
    ('ShareLock'::text, 'ExclusiveLock'::text),
    ('ShareLock'::text, 'AccessExclusiveLock'::text),
    ('ShareRowExclusiveLock'::text, 'RowExclusiveLock'::text),
    ('ShareRowExclusiveLock'::text, 'ShareUpdateExclusiveLock'::text),
    ('ShareRowExclusiveLock'::text, 'ShareLock'::text),
    ('ShareRowExclusiveLock'::text, 'ShareRowExclusiveLock'::text),
    ('ShareRowExclusiveLock'::text, 'ExclusiveLock'::text),
    ('ShareRowExclusiveLock'::text, 'AccessExclusiveLock'::text),
    ('ExclusiveLock'::text, 'RowShareLock'::text),
    ('ExclusiveLock'::text, 'RowExclusiveLock'::text),
    ('ExclusiveLock'::text, 'ShareUpdateExclusiveLock'::text),
    ('ExclusiveLock'::text, 'ShareLock'::text),
    ('ExclusiveLock'::text, 'ShareRowExclusiveLock'::text),
    ('ExclusiveLock'::text, 'ExclusiveLock'::text),
    ('ExclusiveLock'::text, 'AccessExclusiveLock'::text),
    ('AccessExclusiveLock'::text, 'AccessShareLock'::text),
    ('AccessExclusiveLock'::text, 'RowShareLock'::text),
    ('AccessExclusiveLock'::text, 'RowExclusiveLock'::text),
    ('AccessExclusiveLock'::text, 'ShareUpdateExclusiveLock'::text),
    ('AccessExclusiveLock'::text, 'ShareLock'::text),
    ('AccessExclusiveLock'::text, 'ShareRowExclusiveLock'::text),
    ('AccessExclusiveLock'::text, 'ExclusiveLock'::text),
    ('AccessExclusiveLock'::text, 'AccessExclusiveLock'::text)
  )
, lock AS (
  SELECT pid,
     virtualtransaction,
     granted,
     mode,
    (locktype,
     CASE locktype
       WHEN 'relation'      THEN concat_ws(';', 'db:'||datname, 'rel:'||relation::regclass::text)
       WHEN 'extend'        THEN concat_ws(';', 'db:'||datname, 'rel:'||relation::regclass::text)
       WHEN 'page'          THEN concat_ws(';', 'db:'||datname, 'rel:'||relation::regclass::text, 'page#'||page::text)
       WHEN 'tuple'         THEN concat_ws(';', 'db:'||datname, 'rel:'||relation::regclass::text, 'page#'||page::text, 'tuple#'||tuple::text)
       WHEN 'transactionid' THEN transactionid::text
       WHEN 'virtualxid'    THEN virtualxid::text
       WHEN 'object'        THEN concat_ws(';', 'class:'||classid::regclass::text, 'objid:'||objid, 'col#'||objsubid)
       ELSE concat('db:'||datname)
     END::text) AS target
  FROM pg_catalog.pg_locks
  LEFT JOIN pg_catalog.pg_database ON (pg_database.oid = pg_locks.database)
  )
, waiting_lock AS (
  SELECT
    blocker.pid                         AS blocker_pid,
    blocked.pid                         AS pid,
    concat(blocked.mode,blocked.target) AS lock_target
  FROM lock blocker
  JOIN lock blocked
    ON ( NOT blocked.granted
     AND blocker.granted
     AND blocked.pid != blocker.pid
     AND blocked.target IS NOT DISTINCT FROM blocker.target)
  JOIN lock_composite c ON (c.requested = blocked.mode AND c.current = blocker.mode)
  )
, acquired_lock AS (
  WITH waiting AS (
    SELECT lock_target, count(lock_target) AS wait_count FROM waiting_lock GROUP BY lock_target
  )
  SELECT
    pid,
    array_agg(concat(mode,target,' + '||wait_count) ORDER BY wait_count DESC NULLS LAST) AS locks_acquired
  FROM lock
    LEFT JOIN waiting ON waiting.lock_target = concat(mode,target)
  WHERE granted
  GROUP BY pid
  )
, blocking_lock AS (
  SELECT
    ARRAY[date_part('epoch', query_start)::int, pid] AS seq,
     0::int AS depth,
    -1::int AS blocker_pid,
    pid,
    concat('Connect: ',usename,' ',datname,' ',coalesce(host(client_addr)||':'||client_port, 'local')
      , E'\nSQL: ',replace(substr(coalesce(query,'N/A'), 1, 60), E'\n', ' ')
      , E'\nAcquired:\n  '
      , array_to_string(locks_acquired[1:5] ||
                        CASE WHEN array_upper(locks_acquired,1) > 5
                             THEN '... '||(array_upper(locks_acquired,1) - 5)::text||' more ...'
                        END,
                        E'\n  ')
    ) AS lock_info,
    concat(to_char(query_start, CASE WHEN age(query_start) > '24h' THEN 'Day DD Mon' ELSE 'HH24:MI:SS' END),E' started\n'
          ,CASE WHEN wait_event IS NOT NULL THEN 'waiting' ELSE state END,E'\n'
          ,date_trunc('second',age(now(),query_start)),' ago'
    ) AS lock_state
  FROM acquired_lock blocker
  LEFT JOIN pg_stat_activity act USING (pid)
  WHERE EXISTS
         (SELECT 'x' FROM waiting_lock blocked WHERE blocked.blocker_pid = blocker.pid)
    AND NOT EXISTS
         (SELECT 'x' FROM waiting_lock blocked WHERE blocked.pid = blocker.pid)
UNION ALL
  SELECT
    blocker.seq || blocked.pid,
    blocker.depth + 1,
    blocker.pid,
    blocked.pid,
    concat('Connect: ',usename,' ',datname,' ',coalesce(host(client_addr)||':'||client_port, 'local')
      , E'\nSQL: ',replace(substr(coalesce(query,'N/A'), 1, 60), E'\n', ' ')
      , E'\nWaiting: ',blocked.lock_target
      , CASE WHEN locks_acquired IS NOT NULL
             THEN E'\nAcquired:\n  ' ||
                  array_to_string(locks_acquired[1:5] ||
                                  CASE WHEN array_upper(locks_acquired,1) > 5
                                       THEN '... '||(array_upper(locks_acquired,1) - 5)::text||' more ...'
                                  END,
                                  E'\n  ')
        END
    ) AS lock_info,
    concat(to_char(query_start, CASE WHEN age(query_start) > '24h' THEN 'Day DD Mon' ELSE 'HH24:MI:SS' END),E' started\n'
          ,CASE WHEN wait_event IS NOT NULL THEN 'waiting' ELSE state END,E'\n'
          ,date_trunc('second',age(now(),query_start)),' ago'
    ) AS lock_state
  FROM blocking_lock blocker
  JOIN waiting_lock blocked
    ON (blocked.blocker_pid = blocker.pid)
  LEFT JOIN pg_stat_activity act ON (act.pid = blocked.pid)
  LEFT JOIN acquired_lock acq ON (acq.pid = blocked.pid)
  WHERE blocker.depth < 5
  )
SELECT concat(lpad('=> ', 4*depth, ' '),pid::text) AS "PID"
, lock_info AS "Lock Info"
, lock_state AS "State"
FROM blocking_lock
ORDER BY seq;


-- From https://wiki.postgresql.org/wiki/Index_Maintenance

CREATE VIEW duplicate_indexes AS SELECT pg_size_pretty(sum(pg_relation_size(idx))::bigint) as size,
       (array_agg(idx))[1] as idx1, (array_agg(idx))[2] as idx2,
       (array_agg(idx))[3] as idx3, (array_agg(idx))[4] as idx4
FROM (
    SELECT indexrelid::regclass as idx, (indrelid::text ||E'\n'|| indclass::text ||E'\n'|| indkey::text ||E'\n'||
                                         coalesce(indexprs::text,'')||E'\n' || coalesce(indpred::text,'')) as key
    FROM pg_index) sub
GROUP BY key HAVING count(*)>1
ORDER BY sum(pg_relation_size(idx)) DESC;

-- From https://wiki.postgresql.org/wiki/Disk_Usage

CREATE VIEW table_sizes AS WITH RECURSIVE pg_inherit(inhrelid, inhparent) AS
    (select inhrelid, inhparent
    FROM pg_inherits
    UNION
    SELECT child.inhrelid, parent.inhparent
    FROM pg_inherit child, pg_inherits parent
    WHERE child.inhparent = parent.inhrelid),
pg_inherit_short AS (SELECT * FROM pg_inherit WHERE inhparent NOT IN (SELECT inhrelid FROM pg_inherit))
SELECT table_schema
    , TABLE_NAME
    , row_estimate
    , pg_size_pretty(total_bytes) AS total
    , pg_size_pretty(index_bytes) AS INDEX
    , pg_size_pretty(toast_bytes) AS toast
    , pg_size_pretty(table_bytes) AS TABLE
    , total_bytes::float8 / sum(total_bytes) OVER () AS total_size_share
  FROM (
    SELECT *, total_bytes-index_bytes-COALESCE(toast_bytes,0) AS table_bytes
    FROM (
         SELECT c.oid
              , nspname AS table_schema
              , relname AS TABLE_NAME
              , SUM(c.reltuples) OVER (partition BY parent) AS row_estimate
              , SUM(pg_total_relation_size(c.oid)) OVER (partition BY parent) AS total_bytes
              , SUM(pg_indexes_size(c.oid)) OVER (partition BY parent) AS index_bytes
              , SUM(pg_total_relation_size(reltoastrelid)) OVER (partition BY parent) AS toast_bytes
              , parent
          FROM (
                SELECT pg_class.oid
                    , reltuples
                    , relname
                    , relnamespace
                    , pg_class.reltoastrelid
                    , COALESCE(inhparent, pg_class.oid) parent
                FROM pg_class
                    LEFT JOIN pg_inherit_short ON inhrelid = oid
                WHERE relkind IN ('r', 'p')
             ) c
             LEFT JOIN pg_namespace n ON n.oid = c.relnamespace
  ) a
  WHERE oid = parent
) a
ORDER BY total_bytes DESC;

-- From: https://wiki.postgresql.org/wiki/Index_Maintenance

CREATE VIEW index_usage AS SELECT
    t.schemaname,
    t.tablename,
    c.reltuples::bigint                            AS num_rows,
    pg_size_pretty(pg_relation_size(c.oid))        AS table_size,
    psai.indexrelname                              AS index_name,
    pg_size_pretty(pg_relation_size(i.indexrelid)) AS index_size,
    CASE WHEN i.indisunique THEN 'Y' ELSE 'N' END  AS "unique",
    psai.idx_scan                                  AS number_of_scans,
    psai.idx_tup_read                              AS tuples_read,
    psai.idx_tup_fetch                             AS tuples_fetched
FROM
    pg_tables t
    LEFT JOIN pg_class c ON t.tablename = c.relname
    LEFT JOIN pg_index i ON c.oid = i.indrelid
    LEFT JOIN pg_stat_all_indexes psai ON i.indexrelid = psai.indexrelid
WHERE
    t.schemaname NOT IN ('pg_catalog', 'information_schema')
ORDER BY 1, 2;

-- From: https://blog.devgenius.io/top-useful-sql-queries-for-postgresql-35ff3355d265

CREATE VIEW database_sizes AS SELECT pg_database.datname,
       pg_size_pretty(pg_database_size(pg_database.datname)) AS size
FROM pg_database
ORDER BY pg_database_size(pg_database.datname) DESC;

CREATE VIEW schema_sizes AS SELECT A.schemaname,
       pg_size_pretty (SUM(pg_relation_size(C.oid))) as table,
       pg_size_pretty (SUM(pg_total_relation_size(C.oid)-pg_relation_size(C.oid))) as index,
       pg_size_pretty (SUM(pg_total_relation_size(C.oid))) as table_index,
       SUM(n_live_tup)
FROM pg_class C
LEFT JOIN pg_namespace N ON (N.oid = C .relnamespace)
INNER JOIN pg_stat_user_tables A ON C.relname = A.relname
WHERE nspname NOT IN ('pg_catalog', 'information_schema')
AND C .relkind <> 'i'
AND nspname !~ '^pg_toast'
GROUP BY A.schemaname;

CREATE VIEW last_vacuum_analyze AS SELECT relname,
       last_vacuum,
       last_autovacuum
       n_mod_since_analyze,
       last_analyze,
       last_autoanalyze,
       analyze_count,
       autoanalyze_count
    FROM pg_stat_user_tables;

CREATE VIEW table_row_estimates AS SELECT
  schemaname,
  relname,
  n_live_tup
FROM
  pg_stat_user_tables
ORDER BY
  n_live_tup DESC;

-- postgres-meta queries as views from: https://github.com/supabase/postgres-meta

CREATE VIEW pgmeta_columns AS SELECT
  c.oid :: int8 AS table_id,
  nc.nspname AS schema,
  c.relname AS table,
  (c.oid || '.' || a.attnum) AS id,
  a.attnum AS ordinal_position,
  a.attname AS name,
  CASE
    WHEN a.atthasdef THEN pg_get_expr(ad.adbin, ad.adrelid)
    ELSE NULL
  END AS default_value,
  CASE
    WHEN t.typtype = 'd' THEN CASE
      WHEN bt.typelem <> 0 :: oid
      AND bt.typlen = -1 THEN 'ARRAY'
      WHEN nbt.nspname = 'pg_catalog' THEN format_type(t.typbasetype, NULL)
      ELSE 'USER-DEFINED'
    END
    ELSE CASE
      WHEN t.typelem <> 0 :: oid
      AND t.typlen = -1 THEN 'ARRAY'
      WHEN nt.nspname = 'pg_catalog' THEN format_type(a.atttypid, NULL)
      ELSE 'USER-DEFINED'
    END
  END AS data_type,
  COALESCE(bt.typname, t.typname) AS format,
  a.attidentity IN ('a', 'd') AS is_identity,
  CASE
    a.attidentity
    WHEN 'a' THEN 'ALWAYS'
    WHEN 'd' THEN 'BY DEFAULT'
    ELSE NULL
  END AS identity_generation,
  a.attgenerated IN ('s') AS is_generated,
  NOT (
    a.attnotnull
    OR t.typtype = 'd' AND t.typnotnull
  ) AS is_nullable,
  (
    c.relkind IN ('r', 'p')
    OR c.relkind IN ('v', 'f') AND pg_column_is_updatable(c.oid, a.attnum, FALSE)
  ) AS is_updatable,
  uniques.table_id IS NOT NULL AS is_unique,
  array_to_json(
    array(
      SELECT
        enumlabel
      FROM
        pg_catalog.pg_enum enums
      WHERE
        enums.enumtypid = coalesce(bt.oid, t.oid)
        OR enums.enumtypid = coalesce(bt.typelem, t.typelem)
      ORDER BY
        enums.enumsortorder
    )
  ) AS enums,
  col_description(c.oid, a.attnum) AS comment
FROM
  pg_attribute a
  LEFT JOIN pg_attrdef ad ON a.attrelid = ad.adrelid
  AND a.attnum = ad.adnum
  JOIN (
    pg_class c
    JOIN pg_namespace nc ON c.relnamespace = nc.oid
  ) ON a.attrelid = c.oid
  JOIN (
    pg_type t
    JOIN pg_namespace nt ON t.typnamespace = nt.oid
  ) ON a.atttypid = t.oid
  LEFT JOIN (
    pg_type bt
    JOIN pg_namespace nbt ON bt.typnamespace = nbt.oid
  ) ON t.typtype = 'd'
  AND t.typbasetype = bt.oid
  LEFT JOIN (
    SELECT
      conrelid AS table_id,
      conkey[1] AS ordinal_position
    FROM pg_catalog.pg_constraint
    WHERE contype = 'u' AND cardinality(conkey) = 1
  ) AS uniques ON uniques.table_id = c.oid AND uniques.ordinal_position = a.attnum
WHERE
  NOT pg_is_other_temp_schema(nc.oid)
  AND a.attnum > 0
  AND NOT a.attisdropped
  AND (c.relkind IN ('r', 'v', 'm', 'f', 'p'))
  AND (
    pg_has_role(c.relowner, 'USAGE')
    OR has_column_privilege(
      c.oid,
      a.attnum,
      'SELECT, INSERT, UPDATE, REFERENCES'
    )
  );

CREATE VIEW pgmeta_config AS SELECT
  name,
  setting,
  category,
  TRIM(split_part(category, '/', 1)) AS group,
  TRIM(split_part(category, '/', 2)) AS subgroup,
  unit,
  short_desc,
  extra_desc,
  context,
  vartype,
  source,
  min_val,
  max_val,
  enumvals,
  boot_val,
  reset_val,
  sourcefile,
  sourceline,
  pending_restart
FROM
  pg_settings
ORDER BY
  category,
  name;

CREATE VIEW pgmeta_extensions AS SELECT
  e.name,
  n.nspname AS schema,
  e.default_version,
  x.extversion AS installed_version,
  e.comment
FROM
  pg_available_extensions() e(name, default_version, comment)
  LEFT JOIN pg_extension x ON e.name = x.extname
  LEFT JOIN pg_namespace n ON x.extnamespace = n.oid;

CREATE VIEW pgmeta_foreign_tables AS SELECT
  c.oid :: int8 AS id,
  n.nspname AS schema,
  c.relname AS name,
  obj_description(c.oid) AS comment
FROM
  pg_class c
  JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE
  c.relkind = 'f';

CREATE VIEW pgmeta_functions AS with functions as (
  select
    *,
    -- proargmodes is null when all arg modes are IN
    coalesce(
      p.proargmodes,
      array_fill('i'::text, array[cardinality(coalesce(p.proallargtypes, p.proargtypes))])
    ) as arg_modes,
    -- proargnames is null when all args are unnamed
    coalesce(
      p.proargnames,
      array_fill(''::text, array[cardinality(coalesce(p.proallargtypes, p.proargtypes))])
    ) as arg_names,
    -- proallargtypes is null when all arg modes are IN
    coalesce(p.proallargtypes, p.proargtypes) as arg_types,
    array_cat(
      array_fill(false, array[pronargs - pronargdefaults]),
      array_fill(true, array[pronargdefaults])) as arg_has_defaults
  from
    pg_proc as p
  where
    p.prokind = 'f'
)
select
  f.oid::int8 as id,
  n.nspname as schema,
  f.proname as name,
  l.lanname as language,
  case
    when l.lanname = 'internal' then ''
    else f.prosrc
  end as definition,
  case
    when l.lanname = 'internal' then f.prosrc
    else pg_get_functiondef(f.oid)
  end as complete_statement,
  coalesce(f_args.args, '[]') as args,
  pg_get_function_arguments(f.oid) as argument_types,
  pg_get_function_identity_arguments(f.oid) as identity_argument_types,
  f.prorettype::int8 as return_type_id,
  pg_get_function_result(f.oid) as return_type,
  nullif(rt.typrelid::int8, 0) as return_type_relation_id,
  f.proretset as is_set_returning_function,
  case
    when f.provolatile = 'i' then 'IMMUTABLE'
    when f.provolatile = 's' then 'STABLE'
    when f.provolatile = 'v' then 'VOLATILE'
  end as behavior,
  f.prosecdef as security_definer,
  f_config.config_params as config_params
from
  functions f
  left join pg_namespace n on f.pronamespace = n.oid
  left join pg_language l on f.prolang = l.oid
  left join pg_type rt on rt.oid = f.prorettype
  left join (
    select
      oid,
      jsonb_object_agg(param, value) filter (where param is not null) as config_params
    from
      (
        select
          oid,
          (string_to_array(unnest(proconfig), '='))[1] as param,
          (string_to_array(unnest(proconfig), '='))[2] as value
        from
          functions
      ) as t
    group by
      oid
  ) f_config on f_config.oid = f.oid
  left join (
    select
      oid,
      jsonb_agg(jsonb_build_object(
        'mode', t2.mode,
        'name', name,
        'type_id', type_id,
        'has_default', has_default
      )) as args
    from
      (
        select
          oid,
          unnest(arg_modes) as mode,
          unnest(arg_names) as name,
          unnest(arg_types)::int8 as type_id,
          unnest(arg_has_defaults) as has_default
        from
          functions
      ) as t1,
      lateral (
        select
          case
            when t1.mode = 'i' then 'in'
            when t1.mode = 'o' then 'out'
            when t1.mode = 'b' then 'inout'
            when t1.mode = 'v' then 'variadic'
            else 'table'
          end as mode
      ) as t2
    group by
      t1.oid
  ) f_args on f_args.oid = f.oid;

CREATE VIEW pgmeta_materialized_views AS select
  c.oid::int8 as id,
  n.nspname as schema,
  c.relname as name,
  c.relispopulated as is_populated,
  obj_description(c.oid) as comment
from
  pg_class c
  join pg_namespace n on n.oid = c.relnamespace
where
  c.relkind = 'm';

CREATE VIEW pgmeta_policies AS SELECT
  pol.oid :: int8 AS id,
  n.nspname AS schema,
  c.relname AS table,
  c.oid :: int8 AS table_id,
  pol.polname AS name,
  CASE
    WHEN pol.polpermissive THEN 'PERMISSIVE' :: text
    ELSE 'RESTRICTIVE' :: text
  END AS action,
  CASE
    WHEN pol.polroles = '{0}' :: oid [] THEN array_to_json(
      string_to_array('public' :: text, '' :: text) :: name []
    )
    ELSE array_to_json(
      ARRAY(
        SELECT
          pg_roles.rolname
        FROM
          pg_roles
        WHERE
          pg_roles.oid = ANY (pol.polroles)
        ORDER BY
          pg_roles.rolname
      )
    )
  END AS roles,
  CASE
    pol.polcmd
    WHEN 'r' :: "char" THEN 'SELECT' :: text
    WHEN 'a' :: "char" THEN 'INSERT' :: text
    WHEN 'w' :: "char" THEN 'UPDATE' :: text
    WHEN 'd' :: "char" THEN 'DELETE' :: text
    WHEN '*' :: "char" THEN 'ALL' :: text
    ELSE NULL :: text
  END AS command,
  pg_get_expr(pol.polqual, pol.polrelid) AS definition,
  pg_get_expr(pol.polwithcheck, pol.polrelid) AS check
FROM
  pg_policy pol
  JOIN pg_class c ON c.oid = pol.polrelid
  LEFT JOIN pg_namespace n ON n.oid = c.relnamespace;

CREATE VIEW pgmeta_primary_keys AS SELECT
  n.nspname AS schema,
  c.relname AS table_name,
  a.attname AS name,
  c.oid :: int8 AS table_id
FROM
  pg_index i,
  pg_class c,
  pg_attribute a,
  pg_namespace n
WHERE
  i.indrelid = c.oid
  AND c.relnamespace = n.oid
  AND a.attrelid = c.oid
  AND a.attnum = ANY (i.indkey)
  AND i.indisprimary;

CREATE VIEW pgmeta_publications AS SELECT
  p.oid :: int8 AS id,
  p.pubname AS name,
  p.pubowner::regrole::text AS owner,
  p.pubinsert AS publish_insert,
  p.pubupdate AS publish_update,
  p.pubdelete AS publish_delete,
  p.pubtruncate AS publish_truncate,
  CASE
    WHEN p.puballtables THEN NULL
    ELSE pr.tables
  END AS tables
FROM
  pg_catalog.pg_publication AS p
  LEFT JOIN LATERAL (
    SELECT
      COALESCE(
        array_agg(
          json_build_object(
            'id',
            c.oid :: int8,
            'name',
            c.relname,
            'schema',
            nc.nspname
          )
        ),
        '{}'
      ) AS tables
    FROM
      pg_catalog.pg_publication_rel AS pr
      JOIN pg_class AS c ON pr.prrelid = c.oid
      join pg_namespace as nc on c.relnamespace = nc.oid
    WHERE
      pr.prpubid = p.oid
  ) AS pr ON 1 = 1;

CREATE VIEW pgmeta_relationships AS SELECT
  c.oid :: int8 AS id,
  c.conname AS constraint_name,
  nsa.nspname AS source_schema,
  csa.relname AS source_table_name,
  sa.attname AS source_column_name,
  nta.nspname AS target_table_schema,
  cta.relname AS target_table_name,
  ta.attname AS target_column_name
FROM
  pg_constraint c
  JOIN (
    pg_attribute sa
    JOIN pg_class csa ON sa.attrelid = csa.oid
    JOIN pg_namespace nsa ON csa.relnamespace = nsa.oid
  ) ON sa.attrelid = c.conrelid
  AND sa.attnum = ANY (c.conkey)
  JOIN (
    pg_attribute ta
    JOIN pg_class cta ON ta.attrelid = cta.oid
    JOIN pg_namespace nta ON cta.relnamespace = nta.oid
  ) ON ta.attrelid = c.confrelid
  AND ta.attnum = ANY (c.confkey)
WHERE
  c.contype = 'f';

CREATE VIEW pgmeta_roles AS -- TODO: Consider using pg_authid vs. pg_roles for unencrypted password field
SELECT
  oid :: int8 AS id,
  rolname AS name,
  rolsuper AS is_superuser,
  rolcreatedb AS can_create_db,
  rolcreaterole AS can_create_role,
  rolinherit AS inherit_role,
  rolcanlogin AS can_login,
  rolreplication AS is_replication_role,
  rolbypassrls AS can_bypass_rls,
  (
    SELECT
      COUNT(*)
    FROM
      pg_stat_activity
    WHERE
      pg_roles.rolname = pg_stat_activity.usename
  ) AS active_connections,
  CASE WHEN rolconnlimit = -1 THEN current_setting('max_connections') :: int8
       ELSE rolconnlimit
  END AS connection_limit,
  rolpassword AS password,
  rolvaliduntil AS valid_until,
  rolconfig AS config
FROM
  pg_roles;

CREATE VIEW pgmeta_schemas AS select
  n.oid::int8 as id,
  n.nspname as name,
  u.rolname as owner
from
  pg_namespace n,
  pg_roles u
where
  n.nspowner = u.oid
  and (
    pg_has_role(n.nspowner, 'USAGE')
    or has_schema_privilege(n.oid, 'CREATE, USAGE')
  )
  and not pg_catalog.starts_with(n.nspname, 'pg_temp_')
  and not pg_catalog.starts_with(n.nspname, 'pg_toast_temp_');

CREATE VIEW pgmeta_tables AS SELECT
  c.oid :: int8 AS id,
  nc.nspname AS schema,
  c.relname AS name,
  c.relrowsecurity AS rls_enabled,
  c.relforcerowsecurity AS rls_forced,
  CASE
    WHEN c.relreplident = 'd' THEN 'DEFAULT'
    WHEN c.relreplident = 'i' THEN 'INDEX'
    WHEN c.relreplident = 'f' THEN 'FULL'
    ELSE 'NOTHING'
  END AS replica_identity,
  pg_total_relation_size(format('%I.%I', nc.nspname, c.relname)) :: int8 AS bytes,
  pg_size_pretty(
    pg_total_relation_size(format('%I.%I', nc.nspname, c.relname))
  ) AS size,
  pg_stat_get_live_tuples(c.oid) AS live_rows_estimate,
  pg_stat_get_dead_tuples(c.oid) AS dead_rows_estimate,
  obj_description(c.oid) AS comment
FROM
  pg_namespace nc
  JOIN pg_class c ON nc.oid = c.relnamespace
WHERE
  c.relkind IN ('r', 'p')
  AND NOT pg_is_other_temp_schema(nc.oid)
  AND (
    pg_has_role(c.relowner, 'USAGE')
    OR has_table_privilege(
      c.oid,
      'SELECT, INSERT, UPDATE, DELETE, TRUNCATE, REFERENCES, TRIGGER'
    )
    OR has_any_column_privilege(c.oid, 'SELECT, INSERT, UPDATE, REFERENCES')
  );


CREATE VIEW pgmeta_triggers AS SELECT
  pg_t.oid AS id,
  pg_t.tgrelid AS table_id,
  CASE
    WHEN pg_t.tgenabled = 'D' THEN 'DISABLED'
    WHEN pg_t.tgenabled = 'O' THEN 'ORIGIN'
    WHEN pg_t.tgenabled = 'R' THEN 'REPLICA'
    WHEN pg_t.tgenabled = 'A' THEN 'ALWAYS'
  END AS enabled_mode,
  (
    STRING_TO_ARRAY(
      ENCODE(pg_t.tgargs, 'escape'), '\000'
    )
  )[:pg_t.tgnargs] AS function_args,
  is_t.trigger_name AS name,
  is_t.event_object_table AS table,
  is_t.event_object_schema AS schema,
  is_t.action_condition AS condition,
  is_t.action_orientation AS orientation,
  is_t.action_timing AS activation,
  ARRAY_AGG(is_t.event_manipulation)::text[] AS events,
  pg_p.proname AS function_name,
  pg_n.nspname AS function_schema
FROM
  pg_trigger AS pg_t
JOIN
  pg_class AS pg_c
ON pg_t.tgrelid = pg_c.oid
JOIN information_schema.triggers AS is_t
ON is_t.trigger_name = pg_t.tgname
AND pg_c.relname = is_t.event_object_table
JOIN pg_proc AS pg_p
ON pg_t.tgfoid = pg_p.oid
JOIN pg_namespace AS pg_n
ON pg_p.pronamespace = pg_n.oid
GROUP BY
  pg_t.oid,
  pg_t.tgrelid,
  pg_t.tgenabled,
  pg_t.tgargs,
  pg_t.tgnargs,
  is_t.trigger_name,
  is_t.event_object_table,
  is_t.event_object_schema,
  is_t.action_condition,
  is_t.action_orientation,
  is_t.action_timing,
  pg_p.proname,
  pg_n.nspname;


CREATE VIEW pgmeta_types AS select
  t.oid::int8 as id,
  t.typname as name,
  n.nspname as schema,
  format_type (t.oid, null) as format,
  coalesce(t_enums.enums, '[]') as enums,
  coalesce(t_attributes.attributes, '[]') as attributes,
  obj_description (t.oid, 'pg_type') as comment
from
  pg_type t
  left join pg_namespace n on n.oid = t.typnamespace
  left join (
    select
      enumtypid,
      jsonb_agg(enumlabel order by enumsortorder) as enums
    from
      pg_enum
    group by
      enumtypid
  ) as t_enums on t_enums.enumtypid = t.oid
  left join (
    select
      oid,
      jsonb_agg(
        jsonb_build_object('name', a.attname, 'type_id', a.atttypid::int8)
        order by a.attnum asc
      ) as attributes
    from
      pg_class c
      join pg_attribute a on a.attrelid = c.oid
    where
      c.relkind = 'c' and not a.attisdropped
    group by
      c.oid
  ) as t_attributes on t_attributes.oid = t.typrelid
where
  (
    t.typrelid = 0
    or (
      select
        c.relkind = 'c'
      from
        pg_class c
      where
        c.oid = t.typrelid
    )
  );

CREATE VIEW pgmeta_version AS SELECT
  version(),
  current_setting('server_version_num') :: int8 AS version_number,
  (
    SELECT
      COUNT(*) AS active_connections
    FROM
      pg_stat_activity
  ) AS active_connections,
  current_setting('max_connections') :: int8 AS max_connections;

CREATE VIEW pgmeta_views AS SELECT
  c.oid :: int8 AS id,
  n.nspname AS schema,
  c.relname AS name,
  -- See definition of information_schema.views
  (pg_relation_is_updatable(c.oid, false) & 20) = 20 AS is_updatable,
  obj_description(c.oid) AS comment
FROM
  pg_class c
  JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE
  c.relkind = 'v';
$adminpack$,

$description_md$
# michelp@adminpack

A Trusted Language Extension containing a variety of useful databse admin queries.

Postgres database admins are often faced with a variety of issues that
require them introspecting the database state.  This extension
contains a mixed-bag of views that reflect useful information for
admins.

## Installation

Using dbdev:

```
postgres=> select dbdev.install('michelp@adminpack');
 install
---------
 t
(1 row)

postgres=> create schema adminpack;
CREATE SCHEMA
postgres=> create extension "michelp@adminpack" with schema adminpack;
CREATE EXTENSION
```

This will the extension into the `adminpack` schema, substitute with
some other schema if you want it to go somewhere else.

## Index Bloat

Index updates and querying can end up getting slower and slower over
time due to what's called "index bloat".  This is where frequent
updates and deletes can cause space in index data structures to go
unused.  Understanding when this is happening is not obvious, so there
is a rather complex query you can run to detect it.

`index_bloat`

|      Column      |       Type       |
|------------------|------------------|
| current_database | name             |
| schemaname       | name             |
| tblname          | name             |
| idxname          | name             |
| real_size        | numeric          |
| extra_size       | numeric          |
| extra_pct        | double precision |
| fillfactor       | integer          |
| bloat_size       | double precision |
| bloat_pct        | double precision |
| is_na            | boolean          |


## Table Bloat

Like index bloat, tables with high update and delete rates can also
end up containing lots of allocated but unused space.  This query
shows which tables have the most bloat.

`table_bloat`

|      Column      |       Type       |
|------------------|------------------|
| current_database | name             |
| schemaname       | name             |
| tblname          | name             |
| real_size        | numeric          |
| extra_size       | double precision |
| extra_pct        | double precision |
| fillfactor       | integer          |
| bloat_size       | double precision |
| bloat_pct        | double precision |
| is_na            | boolean          |


## Blocking PID Tree

Postgres queries can block each other, and this blocking relationship
can form a tree, where A blocks B, which blocks C, and so on.  This
query formats blocking queries into a tree structure so it's easy to
see what query is causing the blockage.

`blocking_pid_tree`

|  Column   | Type |
|-----------|------|
| PID       | text |
| Lock Info | text |
| State     | text |


## Duplicate Indexes

If a table contains duplicate indexes, then unecessary work is done
updating and storing them, this query will show up to 4 duplicate
indexes per table if they exist.

`duplicate_indexes`

| Column |   Type   |
|--------|----------|
| size   | text     |
| idx1   | regclass |
| idx2   | regclass |
| idx3   | regclass |
| idx4   | regclass |


## Table Sizes

This view shows tables and their sizes.

`table_sizes`

|      Column      |       Type       |
|------------------|------------------|
| table_schema     | name             |
| table_name       | name             |
| row_estimate     | real             |
| total            | text             |
| index            | text             |
| toast            | text             |
| table            | text             |
| total_size_share | double precision |


## Schema Sizes

This view shows schemas and their sizes, which is the sum of the sizes
of all the tables and indexes in the schema.

`schema_sizes`

|   Column    |  Type   |
|-------------|---------|
| schemaname  | name    |
| table       | text    |
| index       | text    |
| table_index | text    |
| sum         | numeric |


## Index Usage

This view shows index size and usage.  An unused index still needs to
be updated and that takes time an storage, so it's a good candidate to
drop.

`index_usage`

|     Column      |  Type  |
|-----------------|--------|
| schemaname      | name   |
| tablename       | name   |
| num_rows        | bigint |
| table_size      | text   |
| index_name      | name   |
| index_size      | text   |
| unique          | text   |
| number_of_scans | bigint |
| tuples_read     | bigint |
| tuples_fetched  | bigint |


## Last Vacuum Analyze

This views shows the last time a table was vacuumed an analyzed.

`last_vacuum_analyze`

|       Column        |           Type           |
|---------------------|--------------------------|
| relname             | name                     |
| last_vacuum         | timestamp with time zone |
| n_mod_since_analyze | timestamp with time zone |
| last_analyze        | timestamp with time zone |
| last_autoanalyze    | timestamp with time zone |
| analyze_count       | bigint                   |
| autoanalyze_count   | bigint                   |

## Table Row Estimates

This view shows estimates for the number of rows in a table.  This is
just an estimate and depends on up to date table statistics.

`table_row_estimates`

|   Column   |  Type  |
|------------|--------|
| schemaname | name   |
| relname    | name   |
| n_live_tup | bigint |

## PGMeta Columns

This view shows infromation for the columns of tables in the system.

`pgmeta_columns`

|       Column        |   Type   |
|---------------------|----------|
| table_id            | bigint   |
| schema              | name     |
| table               | name     |
| id                  | text     |
| ordinal_position    | smallint |
| name                | name     |
| default_value       | text     |
| data_type           | text     |
| format              | name     |
| is_identity         | boolean  |
| identity_generation | text     |
| is_generated        | boolean  |
| is_nullable         | boolean  |
| is_updatable        | boolean  |
| is_unique           | boolean  |
| enums               | json     |
| comment             | text     |

## PGMeta Config

This views shows the configuration of the database.

`pgmeta_config`

|     Column      |  Type   |
|-----------------|---------|
| name            | text    |
| setting         | text    |
| category        | text    |
| group           | text    |
| subgroup        | text    |
| unit            | text    |
| short_desc      | text    |
| extra_desc      | text    |
| context         | text    |
| vartype         | text    |
| source          | text    |
| min_val         | text    |
| max_val         | text    |
| enumvals        | text[]  |
| boot_val        | text    |
| reset_val       | text    |
| sourcefile      | text    |
| sourceline      | integer |
| pending_restart | boolean |

## PGMeta Extensions

This view shows installed extensions in the database.

`pgmeta_extensions`

|      Column       | Type |
|-------------------|------|
| name              | name |
| schema            | name |
| default_version   | text |
| installed_version | text |
| comment           | text |

## PGMeta Foreign Tables

This view shows foreign tables in the database.

`pgmeta_foreign_tables`

| Column  |  Type  |
|---------|--------|
| id      | bigint |
| schema  | name   |
| name    | name   |
| comment | text   |

## PGMeta Functions

This view shows functions in the database.

`pgmeta_functions`

|          Column           |  Type   |
|---------------------------|---------|
| id                        | bigint  |
| schema                    | name    |
| name                      | name    |
| language                  | name    |
| definition                | text    |
| complete_statement        | text    |
| args                      | jsonb   |
| argument_types            | text    |
| identity_argument_types   | text    |
| return_type_id            | bigint  |
| return_type               | text    |
| return_type_relation_id   | bigint  |
| is_set_returning_function | boolean |
| behavior                  | text    |
| security_definer          | boolean |
| config_params             | jsonb   |

## PGMeta Materialized Views

This view shows materialized views in the database.

`pgmeta_materialized_views`

|    Column    |  Type   |
|--------------|---------|
| id           | bigint  |
| schema       | name    |
| name         | name    |
| is_populated | boolean |
| comment      | text    |

## PGMeta Policies

This view shows Row Level Security Policies in the database.

`pgmeta_policies`

|   Column   |  Type  |
|------------|--------|
| id         | bigint |
| schema     | name   |
| table      | name   |
| table_id   | bigint |
| name       | name   |
| action     | text   |
| roles      | json   |
| command    | text   |
| definition | text   |
| check      | text   |

## PGMeta Primary Keys

This view shows primary keys in the database.

`pgmeta_primary_keys`

|   Column   |  Type  |
|------------|--------|
| schema     | name   |
| table_name | name   |
| name       | name   |
| table_id   | bigint |

## PGMeta Publications

This view shows logical replication publishers in the database.

`pgmeta_publications`

|      Column      |  Type   |
|------------------|---------|
| id               | bigint  |
| name             | name    |
| owner            | text    |
| publish_insert   | boolean |
| publish_update   | boolean |
| publish_delete   | boolean |
| publish_truncate | boolean |
| tables           | json[]  |

## PGMeta Relationships

This view shows foreign key relationships in the database.

`pgmeta_relationships`

|       Column        |  Type  |
|---------------------|--------|
| id                  | bigint |
| constraint_name     | name   |
| source_schema       | name   |
| source_table_name   | name   |
| source_column_name  | name   |
| target_table_schema | name   |
| target_table_name   | name   |
| target_column_name  | name   |

## PGMeta Roles

This view shows roles in the database system.  Note that roles are
global objects and apply to all databases.

`pgmeta_roles`

|       Column        |           Type           |
|---------------------|--------------------------|
| id                  | bigint                   |
| name                | name                     |
| is_superuser        | boolean                  |
| can_create_db       | boolean                  |
| can_create_role     | boolean                  |
| inherit_role        | boolean                  |
| can_login           | boolean                  |
| is_replication_role | boolean                  |
| can_bypass_rls      | boolean                  |
| active_connections  | bigint                   |
| connection_limit    | bigint                   |
| password            | text                     |
| valid_until         | timestamp with time zone |
| config              | text[]                   |

## PGMeta Schemas

This view shows all schemas in the database.

`pgmeta_schemas`

| Column |  Type  |
|--------|--------|
| id     | bigint |
| name   | name   |
| owner  | name   |

## PGMeta Tables

This view shows all tables in the database.

`pgmeta_tables`

|       Column       |  Type   |
|--------------------|---------|
| id                 | bigint  |
| schema             | name    |
| name               | name    |
| rls_enabled        | boolean |
| rls_forced         | boolean |
| replica_identity   | text    |
| bytes              | bigint  |
| size               | text    |
| live_rows_estimate | bigint  |
| dead_rows_estimate | bigint  |
| comment            | text    |

## PGMeta Triggers

This view shows all triggers in the database.

`pgmeta_triggers`

|     Column      |               Type                |
|-----------------|-----------------------------------|
| id              | oid                               |
| table_id        | oid                               |
| enabled_mode    | text                              |
| function_args   | text[]                            |
| name            | information_schema.sql_identifier |
| table           | information_schema.sql_identifier |
| schema          | information_schema.sql_identifier |
| condition       | information_schema.character_data |
| orientation     | information_schema.character_data |
| activation      | information_schema.character_data |
| events          | text[]                            |
| function_name   | name                              |
| function_schema | name                              |

## PGMeta Types

This view shows all types in the database.

`pgmeta_types`

|   Column   |  Type  |
|------------|--------|
| id         | bigint |
| name       | name   |
| schema     | name   |
| format     | text   |
| enums      | jsonb  |
| attributes | jsonb  |
| comment    | text   |

## PGMeta Version

This view shows the current database version.

`pgmeta_version`

|       Column       |  Type  |
|--------------------|--------|
| version            | text   |
| version_number     | bigint |
| active_connections | bigint |
| max_connections    | bigint |

## PGMeta Views

This view shows all views in the database.

`pgmeta_views`

|    Column    |  Type   |
|--------------|---------|
| id           | bigint  |
| schema       | name    |
| name         | name    |
| is_updatable | boolean |
| comment      | text    |
$description_md$
);
