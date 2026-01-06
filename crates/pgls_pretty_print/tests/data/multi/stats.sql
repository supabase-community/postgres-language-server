SHOW track_counts;

SELECT backend_type, object, context FROM pg_stat_io
  ORDER BY backend_type COLLATE "C", object COLLATE "C", context COLLATE "C";

SET enable_seqscan TO on;

SET enable_indexscan TO on;

SET enable_indexonlyscan TO off;

SET track_functions TO 'all';

SELECT oid AS dboid from pg_database where datname = current_database() ;

BEGIN;

SET LOCAL stats_fetch_consistency = snapshot;

CREATE TABLE prevstats AS
SELECT t.seq_scan, t.seq_tup_read, t.idx_scan, t.idx_tup_fetch,
       (b.heap_blks_read + b.heap_blks_hit) AS heap_blks,
       (b.idx_blks_read + b.idx_blks_hit) AS idx_blks,
       pg_stat_get_snapshot_timestamp() as snap_ts
  FROM pg_catalog.pg_stat_user_tables AS t,
       pg_catalog.pg_statio_user_tables AS b
 WHERE t.relname='tenk2' AND b.relname='tenk2';

COMMIT;

CREATE TABLE trunc_stats_test(id serial);

CREATE TABLE trunc_stats_test1(id serial, stuff text);

CREATE TABLE trunc_stats_test2(id serial);

CREATE TABLE trunc_stats_test3(id serial, stuff text);

CREATE TABLE trunc_stats_test4(id serial);

INSERT INTO trunc_stats_test DEFAULT VALUES;

INSERT INTO trunc_stats_test DEFAULT VALUES;

INSERT INTO trunc_stats_test DEFAULT VALUES;

TRUNCATE trunc_stats_test;

INSERT INTO trunc_stats_test1 DEFAULT VALUES;

INSERT INTO trunc_stats_test1 DEFAULT VALUES;

INSERT INTO trunc_stats_test1 DEFAULT VALUES;

UPDATE trunc_stats_test1 SET id = id + 10 WHERE id IN (1, 2);

DELETE FROM trunc_stats_test1 WHERE id = 3;

BEGIN;

UPDATE trunc_stats_test1 SET id = id + 100;

TRUNCATE trunc_stats_test1;

INSERT INTO trunc_stats_test1 DEFAULT VALUES;

COMMIT;

BEGIN;

INSERT INTO trunc_stats_test2 DEFAULT VALUES;

INSERT INTO trunc_stats_test2 DEFAULT VALUES;

SAVEPOINT p1;

INSERT INTO trunc_stats_test2 DEFAULT VALUES;

TRUNCATE trunc_stats_test2;

INSERT INTO trunc_stats_test2 DEFAULT VALUES;

RELEASE SAVEPOINT p1;

COMMIT;

BEGIN;

INSERT INTO trunc_stats_test3 DEFAULT VALUES;

INSERT INTO trunc_stats_test3 DEFAULT VALUES;

SAVEPOINT p1;

INSERT INTO trunc_stats_test3 DEFAULT VALUES;

INSERT INTO trunc_stats_test3 DEFAULT VALUES;

TRUNCATE trunc_stats_test3;

INSERT INTO trunc_stats_test3 DEFAULT VALUES;

ROLLBACK TO SAVEPOINT p1;

COMMIT;

BEGIN;

INSERT INTO trunc_stats_test4 DEFAULT VALUES;

INSERT INTO trunc_stats_test4 DEFAULT VALUES;

TRUNCATE trunc_stats_test4;

INSERT INTO trunc_stats_test4 DEFAULT VALUES;

ROLLBACK;

SELECT count(*) FROM tenk2;

SET enable_bitmapscan TO off;

SELECT count(*) FROM tenk2 WHERE unique1 = 1;

RESET enable_bitmapscan;

SELECT pg_stat_force_next_flush();

BEGIN;

SET LOCAL stats_fetch_consistency = snapshot;

SELECT relname, n_tup_ins, n_tup_upd, n_tup_del, n_live_tup, n_dead_tup
  FROM pg_stat_user_tables
 WHERE relname like 'trunc_stats_test%' order by relname;

SELECT st.seq_scan >= pr.seq_scan + 1,
       st.seq_tup_read >= pr.seq_tup_read + cl.reltuples,
       st.idx_scan >= pr.idx_scan + 1,
       st.idx_tup_fetch >= pr.idx_tup_fetch + 1
  FROM pg_stat_user_tables AS st, pg_class AS cl, prevstats AS pr
 WHERE st.relname='tenk2' AND cl.relname='tenk2';

SELECT st.heap_blks_read + st.heap_blks_hit >= pr.heap_blks + cl.relpages,
       st.idx_blks_read + st.idx_blks_hit >= pr.idx_blks + 1
  FROM pg_statio_user_tables AS st, pg_class AS cl, prevstats AS pr
 WHERE st.relname='tenk2' AND cl.relname='tenk2';

SELECT pr.snap_ts < pg_stat_get_snapshot_timestamp() as snapshot_newer
FROM prevstats AS pr;

COMMIT;

CREATE FUNCTION stats_test_func1() RETURNS VOID LANGUAGE plpgsql AS $$BEGIN END;$$;

SELECT 'stats_test_func1()'::regprocedure::oid AS stats_test_func1_oid ;

CREATE FUNCTION stats_test_func2() RETURNS VOID LANGUAGE plpgsql AS $$BEGIN END;$$;

SELECT 'stats_test_func2()'::regprocedure::oid AS stats_test_func2_oid ;

BEGIN;

SET LOCAL stats_fetch_consistency = none;

SELECT pg_stat_get_function_calls('stats_test_func1_oid');

SELECT pg_stat_get_xact_function_calls('stats_test_func1_oid');

SELECT stats_test_func1();

SELECT pg_stat_get_xact_function_calls('stats_test_func1_oid');

SELECT stats_test_func1();

SELECT pg_stat_get_xact_function_calls('stats_test_func1_oid');

SELECT pg_stat_get_function_calls('stats_test_func1_oid');

COMMIT;

BEGIN;

SELECT stats_test_func2();

SAVEPOINT foo;

SELECT stats_test_func2();

ROLLBACK TO SAVEPOINT foo;

SELECT pg_stat_get_xact_function_calls('stats_test_func2_oid');

SELECT stats_test_func2();

COMMIT;

BEGIN;

SELECT stats_test_func2();

ROLLBACK;

SELECT pg_stat_force_next_flush();

SELECT funcname, calls FROM pg_stat_user_functions WHERE funcid = 'stats_test_func1_oid';

SELECT funcname, calls FROM pg_stat_user_functions WHERE funcid = 'stats_test_func2_oid';

BEGIN;

SELECT funcname, calls FROM pg_stat_user_functions WHERE funcid = 'stats_test_func1_oid';

DROP FUNCTION stats_test_func1();

SELECT funcname, calls FROM pg_stat_user_functions WHERE funcid = 'stats_test_func1_oid';

SELECT pg_stat_get_function_calls('stats_test_func1_oid');

ROLLBACK;

SELECT funcname, calls FROM pg_stat_user_functions WHERE funcid = 'stats_test_func1_oid';

SELECT pg_stat_get_function_calls('stats_test_func1_oid');

BEGIN;

DROP FUNCTION stats_test_func1();

COMMIT;

SELECT funcname, calls FROM pg_stat_user_functions WHERE funcid = 'stats_test_func1_oid';

SELECT pg_stat_get_function_calls('stats_test_func1_oid');

BEGIN;

SELECT stats_test_func2();

SAVEPOINT a;

SELECT stats_test_func2();

SAVEPOINT b;

DROP FUNCTION stats_test_func2();

COMMIT;

SELECT funcname, calls FROM pg_stat_user_functions WHERE funcid = 'stats_test_func2_oid';

SELECT pg_stat_get_function_calls('stats_test_func2_oid');

CREATE TABLE drop_stats_test();

INSERT INTO drop_stats_test DEFAULT VALUES;

SELECT 'drop_stats_test'::regclass::oid AS drop_stats_test_oid ;

CREATE TABLE drop_stats_test_xact();

INSERT INTO drop_stats_test_xact DEFAULT VALUES;

SELECT 'drop_stats_test_xact'::regclass::oid AS drop_stats_test_xact_oid ;

CREATE TABLE drop_stats_test_subxact();

INSERT INTO drop_stats_test_subxact DEFAULT VALUES;

SELECT 'drop_stats_test_subxact'::regclass::oid AS drop_stats_test_subxact_oid ;

SELECT pg_stat_force_next_flush();

SELECT pg_stat_get_live_tuples('drop_stats_test_oid');

DROP TABLE drop_stats_test;

SELECT pg_stat_get_live_tuples('drop_stats_test_oid');

SELECT pg_stat_get_xact_tuples_inserted('drop_stats_test_oid');

SELECT pg_stat_get_live_tuples('drop_stats_test_xact_oid');

SELECT pg_stat_get_tuples_inserted('drop_stats_test_xact_oid');

SELECT pg_stat_get_xact_tuples_inserted('drop_stats_test_xact_oid');

BEGIN;

INSERT INTO drop_stats_test_xact DEFAULT VALUES;

SELECT pg_stat_get_xact_tuples_inserted('drop_stats_test_xact_oid');

DROP TABLE drop_stats_test_xact;

SELECT pg_stat_get_xact_tuples_inserted('drop_stats_test_xact_oid');

ROLLBACK;

SELECT pg_stat_force_next_flush();

SELECT pg_stat_get_live_tuples('drop_stats_test_xact_oid');

SELECT pg_stat_get_tuples_inserted('drop_stats_test_xact_oid');

SELECT pg_stat_get_live_tuples('drop_stats_test_xact_oid');

SELECT pg_stat_get_tuples_inserted('drop_stats_test_xact_oid');

BEGIN;

INSERT INTO drop_stats_test_xact DEFAULT VALUES;

SELECT pg_stat_get_xact_tuples_inserted('drop_stats_test_xact_oid');

DROP TABLE drop_stats_test_xact;

SELECT pg_stat_get_xact_tuples_inserted('drop_stats_test_xact_oid');

COMMIT;

SELECT pg_stat_force_next_flush();

SELECT pg_stat_get_live_tuples('drop_stats_test_xact_oid');

SELECT pg_stat_get_tuples_inserted('drop_stats_test_xact_oid');

SELECT pg_stat_get_live_tuples('drop_stats_test_subxact_oid');

BEGIN;

INSERT INTO drop_stats_test_subxact DEFAULT VALUES;

SAVEPOINT sp1;

INSERT INTO drop_stats_test_subxact DEFAULT VALUES;

SELECT pg_stat_get_xact_tuples_inserted('drop_stats_test_subxact_oid');

SAVEPOINT sp2;

DROP TABLE drop_stats_test_subxact;

ROLLBACK TO SAVEPOINT sp2;

SELECT pg_stat_get_xact_tuples_inserted('drop_stats_test_subxact_oid');

COMMIT;

SELECT pg_stat_force_next_flush();

SELECT pg_stat_get_live_tuples('drop_stats_test_subxact_oid');

SELECT pg_stat_get_live_tuples('drop_stats_test_subxact_oid');

BEGIN;

SAVEPOINT sp1;

DROP TABLE drop_stats_test_subxact;

SAVEPOINT sp2;

ROLLBACK TO SAVEPOINT sp1;

COMMIT;

SELECT pg_stat_get_live_tuples('drop_stats_test_subxact_oid');

SELECT pg_stat_get_live_tuples('drop_stats_test_subxact_oid');

BEGIN;

SAVEPOINT sp1;

DROP TABLE drop_stats_test_subxact;

SAVEPOINT sp2;

RELEASE SAVEPOINT sp1;

COMMIT;

SELECT pg_stat_get_live_tuples('drop_stats_test_subxact_oid');

DROP TABLE trunc_stats_test, trunc_stats_test1, trunc_stats_test2, trunc_stats_test3, trunc_stats_test4;

DROP TABLE prevstats;

BEGIN;

CREATE TEMPORARY TABLE test_last_scan(idx_col int primary key, noidx_col int);

INSERT INTO test_last_scan(idx_col, noidx_col) VALUES(1, 1);

SELECT pg_stat_force_next_flush();

SELECT last_seq_scan, last_idx_scan FROM pg_stat_all_tables WHERE relid = 'test_last_scan'::regclass;

COMMIT;

SELECT pg_stat_reset_single_table_counters('test_last_scan'::regclass);

SELECT seq_scan, idx_scan FROM pg_stat_all_tables WHERE relid = 'test_last_scan'::regclass;

BEGIN;

SET LOCAL enable_seqscan TO on;

SET LOCAL enable_indexscan TO on;

SET LOCAL enable_bitmapscan TO off;

SELECT count(*) FROM test_last_scan WHERE noidx_col = 1;

SELECT count(*) FROM test_last_scan WHERE noidx_col = 1;

SET LOCAL enable_seqscan TO off;

SELECT count(*) FROM test_last_scan WHERE idx_col = 1;

SELECT count(*) FROM test_last_scan WHERE idx_col = 1;

SELECT pg_stat_force_next_flush();

COMMIT;

SELECT last_seq_scan AS test_last_seq, last_idx_scan AS test_last_idx
FROM pg_stat_all_tables WHERE relid = 'test_last_scan'::regclass ;

SELECT pg_sleep(0.1);

BEGIN;

SET LOCAL enable_seqscan TO on;

SET LOCAL enable_indexscan TO off;

SET LOCAL enable_bitmapscan TO off;

SELECT count(*) FROM test_last_scan WHERE noidx_col = 1;

SELECT count(*) FROM test_last_scan WHERE noidx_col = 1;

SELECT pg_stat_force_next_flush();

COMMIT;

SELECT seq_scan, 'test_last_seq' < last_seq_scan AS seq_ok, idx_scan, 'test_last_idx' = last_idx_scan AS idx_ok
FROM pg_stat_all_tables WHERE relid = 'test_last_scan'::regclass;

SELECT last_seq_scan AS test_last_seq, last_idx_scan AS test_last_idx
FROM pg_stat_all_tables WHERE relid = 'test_last_scan'::regclass ;

SELECT pg_sleep(0.1);

BEGIN;

SET LOCAL enable_seqscan TO off;

SET LOCAL enable_indexscan TO on;

SET LOCAL enable_bitmapscan TO off;

SELECT count(*) FROM test_last_scan WHERE idx_col = 1;

SELECT count(*) FROM test_last_scan WHERE idx_col = 1;

SELECT pg_stat_force_next_flush();

COMMIT;

SELECT seq_scan, 'test_last_seq' = last_seq_scan AS seq_ok, idx_scan, 'test_last_idx' < last_idx_scan AS idx_ok
FROM pg_stat_all_tables WHERE relid = 'test_last_scan'::regclass;

SELECT last_seq_scan AS test_last_seq, last_idx_scan AS test_last_idx
FROM pg_stat_all_tables WHERE relid = 'test_last_scan'::regclass ;

SELECT pg_sleep(0.1);

BEGIN;

SET LOCAL enable_seqscan TO off;

SET LOCAL enable_indexscan TO off;

SET LOCAL enable_bitmapscan TO on;

SELECT count(*) FROM test_last_scan WHERE idx_col = 1;

SELECT count(*) FROM test_last_scan WHERE idx_col = 1;

SELECT pg_stat_force_next_flush();

COMMIT;

SELECT seq_scan, 'test_last_seq' = last_seq_scan AS seq_ok, idx_scan, 'test_last_idx' < last_idx_scan AS idx_ok
FROM pg_stat_all_tables WHERE relid = 'test_last_scan'::regclass;

SELECT shobj_description(d.oid, 'pg_database') as description_before
  FROM pg_database d WHERE datname = current_database() ;

BEGIN;

SELECT current_database() as datname ;

SELECT pg_stat_force_next_flush();

COMMIT;

SELECT (n_tup_ins + n_tup_upd) > 0 AS has_data FROM pg_stat_all_tables
  WHERE relid = 'pg_shdescription'::regclass;

SELECT pg_stat_reset_single_table_counters('pg_shdescription'::regclass);

SELECT (n_tup_ins + n_tup_upd) > 0 AS has_data FROM pg_stat_all_tables
  WHERE relid = 'pg_shdescription'::regclass;

SELECT sessions AS db_stat_sessions FROM pg_stat_database WHERE datname = (SELECT current_database()) ;

SELECT pg_stat_force_next_flush();

SELECT sessions > 'db_stat_sessions' FROM pg_stat_database WHERE datname = (SELECT current_database());

SELECT num_requested AS rqst_ckpts_before FROM pg_stat_checkpointer ;

SELECT wal_bytes AS wal_bytes_before FROM pg_stat_wal ;

SELECT wal_bytes AS backend_wal_bytes_before from pg_stat_get_backend_wal(pg_backend_pid()) ;

CREATE TEMP TABLE test_stats_temp AS SELECT 17;

DROP TABLE test_stats_temp;

SELECT num_requested > 'rqst_ckpts_before' FROM pg_stat_checkpointer;

SELECT wal_bytes > 'wal_bytes_before' FROM pg_stat_wal;

SELECT pg_stat_force_next_flush();

SELECT wal_bytes > 'backend_wal_bytes_before' FROM pg_stat_get_backend_wal(pg_backend_pid());

SELECT (current_schemas(true))[1] = ('pg_temp_' || beid::text) AS match
FROM pg_stat_get_backend_idset() beid
WHERE pg_stat_get_backend_pid(beid) = pg_backend_pid();

SELECT stats_reset AS slru_commit_ts_reset_ts FROM pg_stat_slru WHERE name = 'commit_timestamp' ;

SELECT stats_reset AS slru_notify_reset_ts FROM pg_stat_slru WHERE name = 'notify' ;

SELECT pg_stat_reset_slru('commit_timestamp');

SELECT stats_reset > 'slru_commit_ts_reset_ts'::timestamptz FROM pg_stat_slru WHERE name = 'commit_timestamp';

SELECT stats_reset AS slru_commit_ts_reset_ts FROM pg_stat_slru WHERE name = 'commit_timestamp' ;

SELECT pg_stat_reset_slru();

SELECT stats_reset > 'slru_commit_ts_reset_ts'::timestamptz FROM pg_stat_slru WHERE name = 'commit_timestamp';

SELECT stats_reset > 'slru_notify_reset_ts'::timestamptz FROM pg_stat_slru WHERE name = 'notify';

SELECT stats_reset AS archiver_reset_ts FROM pg_stat_archiver ;

SELECT pg_stat_reset_shared('archiver');

SELECT stats_reset > 'archiver_reset_ts'::timestamptz FROM pg_stat_archiver;

SELECT stats_reset AS bgwriter_reset_ts FROM pg_stat_bgwriter ;

SELECT pg_stat_reset_shared('bgwriter');

SELECT stats_reset > 'bgwriter_reset_ts'::timestamptz FROM pg_stat_bgwriter;

SELECT stats_reset AS checkpointer_reset_ts FROM pg_stat_checkpointer ;

SELECT pg_stat_reset_shared('checkpointer');

SELECT stats_reset > 'checkpointer_reset_ts'::timestamptz FROM pg_stat_checkpointer;

SELECT stats_reset AS recovery_prefetch_reset_ts FROM pg_stat_recovery_prefetch ;

SELECT pg_stat_reset_shared('recovery_prefetch');

SELECT stats_reset > 'recovery_prefetch_reset_ts'::timestamptz FROM pg_stat_recovery_prefetch;

SELECT max(stats_reset) AS slru_reset_ts FROM pg_stat_slru ;

SELECT pg_stat_reset_shared('slru');

SELECT max(stats_reset) > 'slru_reset_ts'::timestamptz FROM pg_stat_slru;

SELECT stats_reset AS wal_reset_ts FROM pg_stat_wal ;

SELECT pg_stat_reset_shared('wal');

SELECT stats_reset > 'wal_reset_ts'::timestamptz FROM pg_stat_wal;

SELECT pg_stat_reset_shared('unknown');

SELECT pg_stat_reset();

SELECT stats_reset AS db_reset_ts FROM pg_stat_database WHERE datname = (SELECT current_database()) ;

SELECT pg_stat_reset();

SELECT stats_reset > 'db_reset_ts'::timestamptz FROM pg_stat_database WHERE datname = (SELECT current_database());

BEGIN;

SET LOCAL stats_fetch_consistency = snapshot;

SELECT pg_stat_get_snapshot_timestamp();

SELECT pg_stat_get_function_calls(0);

SELECT pg_stat_get_snapshot_timestamp() >= NOW();

SELECT pg_stat_clear_snapshot();

SELECT pg_stat_get_snapshot_timestamp();

COMMIT;

BEGIN;

SET LOCAL stats_fetch_consistency = cache;

SELECT pg_stat_get_function_calls(0);

SELECT pg_stat_get_snapshot_timestamp() IS NOT NULL AS snapshot_ok;

SET LOCAL stats_fetch_consistency = snapshot;

SELECT pg_stat_get_snapshot_timestamp() IS NOT NULL AS snapshot_ok;

SELECT pg_stat_get_function_calls(0);

SELECT pg_stat_get_snapshot_timestamp() IS NOT NULL AS snapshot_ok;

SET LOCAL stats_fetch_consistency = none;

SELECT pg_stat_get_snapshot_timestamp() IS NOT NULL AS snapshot_ok;

SELECT pg_stat_get_function_calls(0);

SELECT pg_stat_get_snapshot_timestamp() IS NOT NULL AS snapshot_ok;

ROLLBACK;

SELECT pg_stat_have_stats('bgwriter', 0, 0);

SELECT pg_stat_have_stats('zaphod', 0, 0);

SELECT pg_stat_have_stats('database', 'dboid', 1);

SELECT pg_stat_have_stats('database', 'dboid', 0);

CREATE table stats_test_tab1 as select generate_series(1,10) a;

CREATE index stats_test_idx1 on stats_test_tab1(a);

SELECT 'stats_test_idx1'::regclass::oid AS stats_test_idx1_oid ;

SET enable_seqscan TO off;

select a from stats_test_tab1 where a = 3;

SELECT pg_stat_have_stats('relation', 'dboid', 'stats_test_idx1_oid');

SELECT pg_stat_have_stats('relation', 'dboid', 'stats_test_idx1_oid');

DROP index stats_test_idx1;

SELECT pg_stat_have_stats('relation', 'dboid', 'stats_test_idx1_oid');

BEGIN;

CREATE index stats_test_idx1 on stats_test_tab1(a);

SELECT 'stats_test_idx1'::regclass::oid AS stats_test_idx1_oid ;

select a from stats_test_tab1 where a = 3;

SELECT pg_stat_have_stats('relation', 'dboid', 'stats_test_idx1_oid');

ROLLBACK;

SELECT pg_stat_have_stats('relation', 'dboid', 'stats_test_idx1_oid');

CREATE index stats_test_idx1 on stats_test_tab1(a);

SELECT 'stats_test_idx1'::regclass::oid AS stats_test_idx1_oid ;

select a from stats_test_tab1 where a = 3;

SELECT pg_stat_have_stats('relation', 'dboid', 'stats_test_idx1_oid');

REINDEX index CONCURRENTLY stats_test_idx1;

SELECT pg_stat_have_stats('relation', 'dboid', 'stats_test_idx1_oid');

SELECT 'stats_test_idx1'::regclass::oid AS stats_test_idx1_oid ;

SELECT pg_stat_have_stats('relation', 'dboid', 'stats_test_idx1_oid');

BEGIN;

SELECT pg_stat_have_stats('relation', 'dboid', 'stats_test_idx1_oid');

DROP index stats_test_idx1;

ROLLBACK;

SELECT pg_stat_have_stats('relation', 'dboid', 'stats_test_idx1_oid');

SET enable_seqscan TO on;

SELECT pg_stat_get_replication_slot(NULL);

SELECT pg_stat_get_subscription_stats(NULL);

SELECT pid AS checkpointer_pid FROM pg_stat_activity
  WHERE backend_type = 'checkpointer' ;

SELECT sum(extends) AS io_sum_shared_before_extends
  FROM pg_stat_io WHERE context = 'normal' AND object = 'relation' ;

SELECT sum(extends) AS my_io_sum_shared_before_extends
  FROM pg_stat_get_backend_io(pg_backend_pid())
  WHERE context = 'normal' AND object = 'relation' ;

SELECT sum(writes) AS writes, sum(fsyncs) AS fsyncs
  FROM pg_stat_io
  WHERE object = 'relation' ;

SELECT sum(writes) AS writes, sum(fsyncs) AS fsyncs
  FROM pg_stat_get_backend_io(pg_backend_pid())
  WHERE object = 'relation' ;

SELECT sum(writes) AS writes, sum(fsyncs) AS fsyncs
  FROM pg_stat_io
  WHERE context = 'normal' AND object = 'wal' ;

CREATE TABLE test_io_shared(a int);

INSERT INTO test_io_shared SELECT i FROM generate_series(1,100)i;

SELECT pg_stat_force_next_flush();

SELECT sum(extends) AS io_sum_shared_after_extends
  FROM pg_stat_io WHERE context = 'normal' AND object = 'relation' ;

SELECT 'io_sum_shared_after_extends' > 'io_sum_shared_before_extends';

SELECT sum(extends) AS my_io_sum_shared_after_extends
  FROM pg_stat_get_backend_io(pg_backend_pid())
  WHERE context = 'normal' AND object = 'relation' ;

SELECT 'my_io_sum_shared_after_extends' > 'my_io_sum_shared_before_extends';

CHECKPOINT;

CHECKPOINT;

SELECT sum(writes) AS writes, sum(fsyncs) AS fsyncs
  FROM pg_stat_io
  WHERE object = 'relation' ;

SELECT 'io_sum_shared_after_writes' > 'io_sum_shared_before_writes';

SELECT current_setting('fsync') = 'off'
  OR 'io_sum_shared_after_fsyncs' > 'io_sum_shared_before_fsyncs';

SELECT sum(writes) AS writes, sum(fsyncs) AS fsyncs
  FROM pg_stat_get_backend_io(pg_backend_pid())
  WHERE object = 'relation' ;

SELECT 'my_io_sum_shared_after_writes' >= 'my_io_sum_shared_before_writes';

SELECT current_setting('fsync') = 'off'
  OR 'my_io_sum_shared_after_fsyncs' >= 'my_io_sum_shared_before_fsyncs';

SELECT sum(writes) AS writes, sum(fsyncs) AS fsyncs
  FROM pg_stat_io
  WHERE context = 'normal' AND object = 'wal' ;

SELECT current_setting('synchronous_commit') = 'on';

SELECT 'io_sum_wal_normal_after_writes' > 'io_sum_wal_normal_before_writes';

SELECT current_setting('fsync') = 'off'
  OR current_setting('wal_sync_method') IN ('open_sync', 'open_datasync')
  OR 'io_sum_wal_normal_after_fsyncs' > 'io_sum_wal_normal_before_fsyncs';

SELECT sum(reads) AS io_sum_shared_before_reads
  FROM pg_stat_io WHERE context = 'normal' AND object = 'relation' ;

BEGIN;

ALTER TABLE test_io_shared SET TABLESPACE regress_tblspace;

SELECT COUNT(*) FROM test_io_shared;

COMMIT;

SELECT pg_stat_force_next_flush();

SELECT sum(reads) AS io_sum_shared_after_reads
  FROM pg_stat_io WHERE context = 'normal' AND object = 'relation'  ;

SELECT 'io_sum_shared_after_reads' > 'io_sum_shared_before_reads';

SELECT sum(hits) AS io_sum_shared_before_hits
  FROM pg_stat_io WHERE context = 'normal' AND object = 'relation' ;

BEGIN;

SET LOCAL enable_nestloop TO on;

SET LOCAL enable_mergejoin TO off;

SET LOCAL enable_hashjoin TO off;

SET LOCAL enable_material TO off;

SELECT COUNT(*) FROM test_io_shared t1 INNER JOIN test_io_shared t2 USING (a);

SELECT COUNT(*) FROM test_io_shared t1 INNER JOIN test_io_shared t2 USING (a);

COMMIT;

SELECT pg_stat_force_next_flush();

SELECT sum(hits) AS io_sum_shared_after_hits
  FROM pg_stat_io WHERE context = 'normal' AND object = 'relation' ;

SELECT 'io_sum_shared_after_hits' > 'io_sum_shared_before_hits';

DROP TABLE test_io_shared;

SET temp_buffers TO 100;

CREATE TEMPORARY TABLE test_io_local(a int, b TEXT);

SELECT sum(extends) AS extends, sum(evictions) AS evictions, sum(writes) AS writes
  FROM pg_stat_io
  WHERE context = 'normal' AND object = 'temp relation' ;

INSERT INTO test_io_local SELECT generate_series(1, 5000) as id, repeat('a', 200);

SELECT pg_relation_size('test_io_local') / current_setting('block_size')::int8 > 100;

SELECT sum(reads) AS io_sum_local_before_reads
  FROM pg_stat_io WHERE context = 'normal' AND object = 'temp relation' ;

SELECT COUNT(*) FROM test_io_local;

SELECT pg_stat_force_next_flush();

SELECT sum(evictions) AS evictions,
       sum(reads) AS reads,
       sum(writes) AS writes,
       sum(extends) AS extends
  FROM pg_stat_io
  WHERE context = 'normal' AND object = 'temp relation'  ;

SELECT 'io_sum_local_after_evictions' > 'io_sum_local_before_evictions',
       'io_sum_local_after_reads' > 'io_sum_local_before_reads',
       'io_sum_local_after_writes' > 'io_sum_local_before_writes',
       'io_sum_local_after_extends' > 'io_sum_local_before_extends';

ALTER TABLE test_io_local SET TABLESPACE regress_tblspace;

SELECT pg_stat_force_next_flush();

SELECT sum(writes) AS io_sum_local_new_tblspc_writes
  FROM pg_stat_io WHERE context = 'normal' AND object = 'temp relation'  ;

SELECT 'io_sum_local_new_tblspc_writes' > 'io_sum_local_after_writes';

RESET temp_buffers;

SET wal_skip_threshold = '1 kB';

SELECT sum(reuses) AS reuses, sum(reads) AS reads, sum(evictions) AS evictions
  FROM pg_stat_io WHERE context = 'vacuum' ;

CREATE TABLE test_io_vac_strategy(a int, b int) WITH (autovacuum_enabled = 'false');

INSERT INTO test_io_vac_strategy SELECT i, i from generate_series(1, 4500)i;

VACUUM (FULL) test_io_vac_strategy;

VACUUM (PARALLEL 0, BUFFER_USAGE_LIMIT 128) test_io_vac_strategy;

SELECT pg_stat_force_next_flush();

SELECT sum(reuses) AS reuses, sum(reads) AS reads, sum(evictions) AS evictions
  FROM pg_stat_io WHERE context = 'vacuum' ;

SELECT 'io_sum_vac_strategy_after_reads' > 'io_sum_vac_strategy_before_reads';

SELECT ('io_sum_vac_strategy_after_reuses' + 'io_sum_vac_strategy_after_evictions') >
  ('io_sum_vac_strategy_before_reuses' + 'io_sum_vac_strategy_before_evictions');

RESET wal_skip_threshold;

SELECT sum(extends) AS io_sum_bulkwrite_strategy_extends_before
  FROM pg_stat_io WHERE context = 'bulkwrite' ;

CREATE TABLE test_io_bulkwrite_strategy AS SELECT i FROM generate_series(1,100)i;

SELECT pg_stat_force_next_flush();

SELECT sum(extends) AS io_sum_bulkwrite_strategy_extends_after
  FROM pg_stat_io WHERE context = 'bulkwrite' ;

SELECT 'io_sum_bulkwrite_strategy_extends_after' > 'io_sum_bulkwrite_strategy_extends_before';

SELECT pg_stat_have_stats('io', 0, 0);

SELECT sum(evictions) + sum(reuses) + sum(extends) + sum(fsyncs) + sum(reads) + sum(writes) + sum(writebacks) + sum(hits) AS io_stats_pre_reset
  FROM pg_stat_io ;

SELECT sum(evictions) + sum(reuses) + sum(extends) + sum(fsyncs) + sum(reads) + sum(writes) + sum(writebacks) + sum(hits) AS my_io_stats_pre_reset
  FROM pg_stat_get_backend_io(pg_backend_pid()) ;

SELECT pg_stat_reset_shared('io');

SELECT sum(evictions) + sum(reuses) + sum(extends) + sum(fsyncs) + sum(reads) + sum(writes) + sum(writebacks) + sum(hits) AS io_stats_post_reset
  FROM pg_stat_io ;

SELECT 'io_stats_post_reset' < 'io_stats_pre_reset';

SELECT sum(evictions) + sum(reuses) + sum(extends) + sum(fsyncs) + sum(reads) + sum(writes) + sum(writebacks) + sum(hits) AS my_io_stats_post_reset
  FROM pg_stat_get_backend_io(pg_backend_pid()) ;

SELECT 'my_io_stats_pre_reset' <= 'my_io_stats_post_reset';

SELECT pg_stat_reset_backend_stats(pg_backend_pid());

SELECT sum(evictions) + sum(reuses) + sum(extends) + sum(fsyncs) + sum(reads) + sum(writes) + sum(writebacks) + sum(hits) AS my_io_stats_post_backend_reset
  FROM pg_stat_get_backend_io(pg_backend_pid()) ;

SELECT 'my_io_stats_pre_reset' > 'my_io_stats_post_backend_reset';

SELECT pg_stat_get_backend_io(NULL);

SELECT pg_stat_get_backend_io(0);

SELECT pg_stat_get_backend_io('checkpointer_pid');

CREATE TABLE brin_hot (
  id  integer PRIMARY KEY,
  val integer NOT NULL
) WITH (autovacuum_enabled = off, fillfactor = 70);

INSERT INTO brin_hot SELECT *, 0 FROM generate_series(1, 235);

CREATE INDEX val_brin ON brin_hot using brin(val);

CREATE FUNCTION wait_for_hot_stats() RETURNS void AS $$
DECLARE
  start_time timestamptz := clock_timestamp();
  updated bool;
BEGIN
  -- we don't want to wait forever; loop will exit after 30 seconds
  FOR i IN 1 .. 300 LOOP
    SELECT (pg_stat_get_tuples_hot_updated('brin_hot'::regclass::oid) > 0) INTO updated;
    EXIT WHEN updated;

    -- wait a little
    PERFORM pg_sleep_for('100 milliseconds');
    -- reset stats snapshot so we can test again
    PERFORM pg_stat_clear_snapshot();
  END LOOP;
  -- report time waited in postmaster log (where it won't change test output)
  RAISE log 'wait_for_hot_stats delayed % seconds',
    EXTRACT(epoch FROM clock_timestamp() - start_time);
END
$$ LANGUAGE plpgsql;

UPDATE brin_hot SET val = -3 WHERE id = 42;

SELECT wait_for_hot_stats();

SELECT pg_stat_get_tuples_hot_updated('brin_hot'::regclass::oid);

DROP TABLE brin_hot;

DROP FUNCTION wait_for_hot_stats();

CREATE TABLE brin_hot_2 (a int, b int);

INSERT INTO brin_hot_2 VALUES (1, 100);

CREATE INDEX ON brin_hot_2 USING brin (b) WHERE a = 2;

UPDATE brin_hot_2 SET a = 2;

SELECT * FROM brin_hot_2 WHERE a = 2 AND b = 100;

SELECT COUNT(*) FROM brin_hot_2 WHERE a = 2 AND b = 100;

SET enable_seqscan = off;

SELECT * FROM brin_hot_2 WHERE a = 2 AND b = 100;

SELECT COUNT(*) FROM brin_hot_2 WHERE a = 2 AND b = 100;

DROP TABLE brin_hot_2;

CREATE TABLE brin_hot_3 (a int, filler text) WITH (fillfactor = 10);

INSERT INTO brin_hot_3 SELECT 1, repeat(' ', 500) FROM generate_series(1, 20);

CREATE INDEX ON brin_hot_3 USING brin (a) WITH (pages_per_range = 1);

UPDATE brin_hot_3 SET a = 2;

SELECT * FROM brin_hot_3 WHERE a = 2;

SELECT COUNT(*) FROM brin_hot_3 WHERE a = 2;

DROP TABLE brin_hot_3;

SET enable_seqscan = on;

CREATE TABLE table_fillfactor (
  n char(1000)
) with (fillfactor=10, autovacuum_enabled=off);

INSERT INTO table_fillfactor
SELECT 'x' FROM generate_series(1,1000);

SELECT * FROM check_estimated_rows('SELECT * FROM table_fillfactor');

DROP TABLE table_fillfactor;
