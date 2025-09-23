CREATE FUNCTION explain_mask_costs(query text, do_analyze bool,
    hide_costs bool, hide_row_est bool, hide_width bool) RETURNS setof text
LANGUAGE plpgsql AS
$$
DECLARE
    ln text;
    analyze_str text;
BEGIN
    IF do_analyze = true THEN
        analyze_str := 'on';
    ELSE
        analyze_str := 'off';
    END IF;

    -- avoid jit related output by disabling it
    SET LOCAL jit = 0;

    FOR ln IN
        EXECUTE format('explain (analyze %s, costs on, summary off, timing off, buffers off) %s',
            analyze_str, query)
    LOOP
        IF hide_costs = true THEN
            ln := regexp_replace(ln, 'cost=\d+\.\d\d\.\.\d+\.\d\d', 'cost=N..N');
        END IF;

        IF hide_row_est = true THEN
            -- don't use 'g' so that we leave the actual rows intact
            ln := regexp_replace(ln, 'rows=\d+', 'rows=N');
        END IF;

        IF hide_width = true THEN
            ln := regexp_replace(ln, 'width=\d+', 'width=N');
        END IF;

        RETURN NEXT ln;
    END LOOP;
END;
$$;

SELECT num_nonnulls(NULL);

SELECT num_nonnulls('1');

SELECT num_nonnulls(NULL::text);

SELECT num_nonnulls(NULL::text, NULL::int);

SELECT num_nonnulls(1, 2, NULL::text, NULL::point, '', int8 '9', 1.0 / NULL);

SELECT num_nonnulls(VARIADIC '{1,2,NULL,3}'::int[]);

SELECT num_nonnulls(VARIADIC '{"1","2","3","4"}'::text[]);

SELECT num_nonnulls(VARIADIC ARRAY(SELECT CASE WHEN i <> 40 THEN i END FROM generate_series(1, 100) i));

SELECT num_nulls(NULL);

SELECT num_nulls('1');

SELECT num_nulls(NULL::text);

SELECT num_nulls(NULL::text, NULL::int);

SELECT num_nulls(1, 2, NULL::text, NULL::point, '', int8 '9', 1.0 / NULL);

SELECT num_nulls(VARIADIC '{1,2,NULL,3}'::int[]);

SELECT num_nulls(VARIADIC '{"1","2","3","4"}'::text[]);

SELECT num_nulls(VARIADIC ARRAY(SELECT CASE WHEN i <> 40 THEN i END FROM generate_series(1, 100) i));

SELECT num_nonnulls(VARIADIC NULL::text[]);

SELECT num_nonnulls(VARIADIC '{}'::int[]);

SELECT num_nulls(VARIADIC NULL::text[]);

SELECT num_nulls(VARIADIC '{}'::int[]);

SELECT num_nonnulls();

SELECT num_nulls();

CREATE FUNCTION test_canonicalize_path(text)
   RETURNS text
   AS 'regresslib'
   LANGUAGE C STRICT IMMUTABLE;

SELECT test_canonicalize_path('/');

SELECT test_canonicalize_path('/./abc/def/');

SELECT test_canonicalize_path('/./../abc/def');

SELECT test_canonicalize_path('/./../../abc/def/');

SELECT test_canonicalize_path('/abc/.././def/ghi');

SELECT test_canonicalize_path('/abc/./../def/ghi//');

SELECT test_canonicalize_path('/abc/def/../..');

SELECT test_canonicalize_path('/abc/def/../../..');

SELECT test_canonicalize_path('/abc/def/../../../../ghi/jkl');

SELECT test_canonicalize_path('.');

SELECT test_canonicalize_path('./');

SELECT test_canonicalize_path('./abc/..');

SELECT test_canonicalize_path('abc/../');

SELECT test_canonicalize_path('abc/../def');

SELECT test_canonicalize_path('..');

SELECT test_canonicalize_path('../abc/def');

SELECT test_canonicalize_path('../abc/..');

SELECT test_canonicalize_path('../abc/../def');

SELECT test_canonicalize_path('../abc/../../def/ghi');

SELECT test_canonicalize_path('./abc/./def/.');

SELECT test_canonicalize_path('./abc/././def/.');

SELECT test_canonicalize_path('./abc/./def/.././ghi/../../../jkl/mno');

SELECT pg_log_backend_memory_contexts(pg_backend_pid());

SELECT pg_log_backend_memory_contexts(pid) FROM pg_stat_activity
  WHERE backend_type = 'checkpointer';

CREATE ROLE regress_log_memory;

SELECT has_function_privilege('regress_log_memory',
  'pg_log_backend_memory_contexts(integer)', 'EXECUTE');

GRANT EXECUTE ON FUNCTION pg_log_backend_memory_contexts(integer)
  TO regress_log_memory;

SELECT has_function_privilege('regress_log_memory',
  'pg_log_backend_memory_contexts(integer)', 'EXECUTE');

SET ROLE regress_log_memory;

SELECT pg_log_backend_memory_contexts(pg_backend_pid());

RESET ROLE;

REVOKE EXECUTE ON FUNCTION pg_log_backend_memory_contexts(integer)
  FROM regress_log_memory;

DROP ROLE regress_log_memory;

select setting as segsize
from pg_settings where name = 'wal_segment_size'

select count(*) > 0 as ok from pg_ls_waldir();

select count(*) > 0 as ok from (select pg_ls_waldir()) ss;

select * from pg_ls_waldir() limit 0;

select count(*) > 0 as ok from (select * from pg_ls_waldir() limit 1) ss;

select (w).size = 'segsize' as ok
from (select pg_ls_waldir() w) ss where length((w).name) = 24 limit 1;

select count(*) >= 0 as ok from pg_ls_archive_statusdir();

select count(*) >= 0 as ok from pg_ls_summariesdir();

select length(pg_read_file('postmaster.pid')) > 20;

select length(pg_read_file('postmaster.pid', 1, 20));

select pg_read_file('does not exist');

select pg_read_file('does not exist', true) IS NULL;

select pg_read_file('does not exist', 0, -1);

select pg_read_file('does not exist', 0, -1, true);

select length(pg_read_binary_file('postmaster.pid')) > 20;

select length(pg_read_binary_file('postmaster.pid', 1, 20));

select pg_read_binary_file('does not exist');

select pg_read_binary_file('does not exist', true) IS NULL;

select pg_read_binary_file('does not exist', 0, -1);

select pg_read_binary_file('does not exist', 0, -1, true);

select size > 20, isdir from pg_stat_file('postmaster.pid');

select * from (select pg_ls_dir('.') a) a where a = 'base' limit 1;

select pg_ls_dir('does not exist', false, false);

select pg_ls_dir('does not exist', true, false);

select count(*) = 1 as dot_found
  from pg_ls_dir('.', false, true) as ls where ls = '.';

select count(*) = 1 as dot_found
  from pg_ls_dir('.', false, false) as ls where ls = '.';

select * from (select (pg_timezone_names()).name) ptn where name='UTC' limit 1;

select count(*) > 0 from
  (select pg_tablespace_databases(oid) as pts from pg_tablespace
   where spcname = 'pg_default') pts
  join pg_database db on pts.pts = db.oid;

CREATE ROLE regress_slot_dir_funcs;

SELECT has_function_privilege('regress_slot_dir_funcs',
  'pg_ls_logicalsnapdir()', 'EXECUTE');

SELECT has_function_privilege('regress_slot_dir_funcs',
  'pg_ls_logicalmapdir()', 'EXECUTE');

SELECT has_function_privilege('regress_slot_dir_funcs',
  'pg_ls_replslotdir(text)', 'EXECUTE');

GRANT pg_monitor TO regress_slot_dir_funcs;

SELECT has_function_privilege('regress_slot_dir_funcs',
  'pg_ls_logicalsnapdir()', 'EXECUTE');

SELECT has_function_privilege('regress_slot_dir_funcs',
  'pg_ls_logicalmapdir()', 'EXECUTE');

SELECT has_function_privilege('regress_slot_dir_funcs',
  'pg_ls_replslotdir(text)', 'EXECUTE');

DROP ROLE regress_slot_dir_funcs;

CREATE FUNCTION my_int_eq(int, int) RETURNS bool
  LANGUAGE internal STRICT IMMUTABLE PARALLEL SAFE
  AS $$int4eq$$;

SELECT * FROM tenk1 a JOIN tenk1 b ON a.unique1 = b.unique1
WHERE my_int_eq(a.unique2, 42);

CREATE FUNCTION test_support_func(internal)
    RETURNS internal
    AS 'regresslib', 'test_support_func'
    LANGUAGE C STRICT;

ALTER FUNCTION my_int_eq(int, int) SUPPORT test_support_func;

SELECT * FROM tenk1 a JOIN tenk1 b ON a.unique1 = b.unique1
WHERE my_int_eq(a.unique2, 42);

CREATE FUNCTION my_gen_series(int, int) RETURNS SETOF integer
  LANGUAGE internal STRICT IMMUTABLE PARALLEL SAFE
  AS $$generate_series_int4$$
  SUPPORT test_support_func;

SELECT * FROM tenk1 a JOIN my_gen_series(1,1000) g ON a.unique1 = g;

SELECT * FROM tenk1 a JOIN my_gen_series(1,10) g ON a.unique1 = g;

SELECT explain_mask_costs($$
SELECT * FROM generate_series(TIMESTAMPTZ '2024-02-01', TIMESTAMPTZ '2024-03-01', INTERVAL '1 day') g(s);$$,
true, true, false, true);

SELECT explain_mask_costs($$
SELECT * FROM generate_series(TIMESTAMP '2024-02-01', TIMESTAMP '2024-03-01', INTERVAL '1 day') g(s);$$,
true, true, false, true);

SELECT explain_mask_costs($$
SELECT * FROM generate_series(TIMESTAMPTZ '2024-02-01', TIMESTAMPTZ '2024-03-01', INTERVAL '1 day', 'UTC') g(s);$$,
true, true, false, true);

SELECT explain_mask_costs($$
SELECT * FROM generate_series(TIMESTAMPTZ '2024-02-01', TIMESTAMPTZ '2024-03-01', INTERVAL '7 day') g(s);$$,
true, true, false, true);

SELECT explain_mask_costs($$
SELECT * FROM generate_series(TIMESTAMPTZ '2024-03-01', TIMESTAMPTZ '2024-02-01', INTERVAL '-1 day') g(s);$$,
true, true, false, true);

SELECT explain_mask_costs($$
SELECT * FROM generate_series(TIMESTAMPTZ '2024-03-01', TIMESTAMPTZ '2024-02-01', INTERVAL '1 day') g(s);$$,
true, true, false, true);

SELECT explain_mask_costs($$
SELECT * FROM generate_series(TIMESTAMPTZ '-infinity', TIMESTAMPTZ 'infinity', INTERVAL '1 day') g(s);$$,
false, true, false, true);

SELECT * FROM generate_series(TIMESTAMPTZ '2024-02-01', TIMESTAMPTZ '2024-03-01', INTERVAL '0 day') g(s);

SELECT explain_mask_costs($$
SELECT * FROM generate_series(1.0, 25.0) g(s);$$,
true, true, false, true);

SELECT explain_mask_costs($$
SELECT * FROM generate_series(1.0, 25.0, 2.0) g(s);$$,
true, true, false, true);

SELECT explain_mask_costs($$
SELECT * FROM generate_series(25.0, 1.0, -1.0) g(s);$$,
true, true, false, true);

SELECT explain_mask_costs($$
SELECT * FROM generate_series(25.0, 1.0, 1.0) g(s);$$,
true, true, false, true);

SELECT explain_mask_costs($$
SELECT * FROM generate_series('-infinity'::NUMERIC, 'infinity'::NUMERIC, 1.0) g(s);$$,
false, true, false, true);

SELECT explain_mask_costs($$
SELECT * FROM generate_series(1.0, 25.0, 'NaN'::NUMERIC) g(s);$$,
false, true, false, true);

SELECT explain_mask_costs($$
SELECT * FROM generate_series(25.0, 2.0, 0.0) g(s);$$,
false, true, false, true);

SELECT count(*) > 0 AS ok FROM pg_control_checkpoint();

SELECT count(*) > 0 AS ok FROM pg_control_init();

SELECT count(*) > 0 AS ok FROM pg_control_recovery();

SELECT count(*) > 0 AS ok FROM pg_control_system();

SELECT * FROM pg_split_walfile_name(NULL);

SELECT * FROM pg_split_walfile_name('invalid');

SELECT segment_number > 0 AS ok_segment_number, timeline_id
  FROM pg_split_walfile_name('000000010000000100000000');

SELECT segment_number > 0 AS ok_segment_number, timeline_id
  FROM pg_split_walfile_name('ffffffFF00000001000000af');

SELECT setting::int8 AS segment_size
FROM pg_settings
WHERE name = 'wal_segment_size'

SELECT segment_number, file_offset
FROM pg_walfile_name_offset('0/0'::pg_lsn + 'segment_size'),
     pg_split_walfile_name(file_name);

SELECT segment_number, file_offset
FROM pg_walfile_name_offset('0/0'::pg_lsn + 'segment_size' + 1),
     pg_split_walfile_name(file_name);

SELECT segment_number, file_offset = 'segment_size' - 1
FROM pg_walfile_name_offset('0/0'::pg_lsn + 'segment_size' - 1),
     pg_split_walfile_name(file_name);

CREATE ROLE regress_current_logfile;

SELECT has_function_privilege('regress_current_logfile',
  'pg_current_logfile()', 'EXECUTE');

GRANT pg_monitor TO regress_current_logfile;

SELECT has_function_privilege('regress_current_logfile',
  'pg_current_logfile()', 'EXECUTE');

DROP ROLE regress_current_logfile;

CREATE TABLE test_chunk_id (a TEXT, b TEXT STORAGE EXTERNAL);

INSERT INTO test_chunk_id VALUES ('x', repeat('x', 8192));

SELECT t.relname AS toastrel FROM pg_class c
  LEFT JOIN pg_class t ON c.reltoastrelid = t.oid
  WHERE c.relname = 'test_chunk_id'

DROP TABLE test_chunk_id;

DROP FUNCTION explain_mask_costs(text, bool, bool, bool, bool);

SELECT gist_translate_cmptype_common(7);

SELECT gist_translate_cmptype_common(3);

CREATE FUNCTION test_relpath()
    RETURNS void
    AS 'regresslib'
    LANGUAGE C;

SELECT test_relpath();

SELECT pg_replication_origin_create('regress_' || repeat('a', 505));
