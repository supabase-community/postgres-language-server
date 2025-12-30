CREATE TABLE vactst (i INT);

INSERT INTO vactst VALUES (1);

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst VALUES (0);

SELECT count(*) FROM vactst;

DELETE FROM vactst WHERE i != 0;

SELECT * FROM vactst;

VACUUM FULL vactst;

UPDATE vactst SET i = i + 1;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst SELECT * FROM vactst;

INSERT INTO vactst VALUES (0);

SELECT count(*) FROM vactst;

DELETE FROM vactst WHERE i != 0;

VACUUM (FULL) vactst;

DELETE FROM vactst;

SELECT * FROM vactst;

VACUUM (FULL, FREEZE) vactst;

VACUUM (ANALYZE, FULL) vactst;

CREATE TABLE vaccluster (i INT PRIMARY KEY);

ALTER TABLE vaccluster CLUSTER ON vaccluster_pkey;

CLUSTER vaccluster;

CREATE FUNCTION do_analyze() RETURNS VOID VOLATILE LANGUAGE SQL
	AS 'ANALYZE pg_am';

CREATE FUNCTION wrap_do_analyze(c INT) RETURNS INT IMMUTABLE LANGUAGE SQL
	AS 'SELECT $1 FROM public.do_analyze()';

CREATE INDEX ON vaccluster(wrap_do_analyze(i));

INSERT INTO vaccluster VALUES (1), (2);

ANALYZE vaccluster;

INSERT INTO vactst SELECT generate_series(1, 300);

DELETE FROM vactst WHERE i % 7 = 0;

BEGIN;

INSERT INTO vactst SELECT generate_series(301, 400);

DELETE FROM vactst WHERE i % 5 <> 0;

ANALYZE vactst;

COMMIT;

BEGIN;

CREATE TABLE past_inh_parent ();

CREATE TABLE past_inh_child () INHERITS (past_inh_parent);

INSERT INTO past_inh_child DEFAULT VALUES;

INSERT INTO past_inh_child DEFAULT VALUES;

ANALYZE past_inh_parent;

SELECT reltuples, relhassubclass
  FROM pg_class WHERE oid = 'past_inh_parent'::regclass;

DROP TABLE past_inh_child;

ANALYZE past_inh_parent;

SELECT reltuples, relhassubclass
  FROM pg_class WHERE oid = 'past_inh_parent'::regclass;

COMMIT;

BEGIN;

CREATE TABLE past_parted (i int) PARTITION BY LIST(i);

CREATE TABLE past_part PARTITION OF past_parted FOR VALUES IN (1);

INSERT INTO past_parted VALUES (1),(1);

ANALYZE past_parted;

DROP TABLE past_part;

SELECT reltuples, relhassubclass
  FROM pg_class WHERE oid = 'past_parted'::regclass;

ANALYZE past_parted;

SELECT reltuples, relhassubclass
  FROM pg_class WHERE oid = 'past_parted'::regclass;

COMMIT;

VACUUM FULL pg_am;

VACUUM FULL pg_class;

VACUUM FULL pg_database;

VACUUM FULL vaccluster;

VACUUM FULL vactst;

VACUUM (DISABLE_PAGE_SKIPPING) vaccluster;

CREATE TABLE pvactst (i INT, a INT[], p POINT) with (autovacuum_enabled = off);

INSERT INTO pvactst SELECT i, array[1,2,3], point(i, i+1) FROM generate_series(1,1000) i;

CREATE INDEX btree_pvactst ON pvactst USING btree (i);

CREATE INDEX hash_pvactst ON pvactst USING hash (i);

CREATE INDEX brin_pvactst ON pvactst USING brin (i);

CREATE INDEX gin_pvactst ON pvactst USING gin (a);

CREATE INDEX gist_pvactst ON pvactst USING gist (p);

CREATE INDEX spgist_pvactst ON pvactst USING spgist (p);

CREATE TABLE pvactst2 (i INT) WITH (autovacuum_enabled = off);

INSERT INTO pvactst2 SELECT generate_series(1, 1000);

CREATE INDEX ON pvactst2 (i);

CREATE INDEX ON pvactst2 (i);

SET min_parallel_index_scan_size to 0;

VACUUM (PARALLEL 2) pvactst;

UPDATE pvactst SET i = i WHERE i < 1000;

VACUUM (PARALLEL 2) pvactst;

UPDATE pvactst SET i = i WHERE i < 1000;

VACUUM (PARALLEL 0) pvactst;

VACUUM (PARALLEL -1) pvactst;

VACUUM (PARALLEL 2, INDEX_CLEANUP FALSE) pvactst;

VACUUM (PARALLEL 2, FULL TRUE) pvactst;

VACUUM (PARALLEL) pvactst;

SET maintenance_work_mem TO 64;

VACUUM (PARALLEL 2) pvactst2;

DELETE FROM pvactst2 WHERE i < 1000;

VACUUM (PARALLEL 2) pvactst2;

RESET maintenance_work_mem;

CREATE TEMPORARY TABLE tmp (a int PRIMARY KEY);

CREATE INDEX tmp_idx1 ON tmp (a);

VACUUM (PARALLEL 1, FULL FALSE) tmp;

VACUUM (PARALLEL 0, FULL TRUE) tmp;

RESET min_parallel_index_scan_size;

DROP TABLE pvactst;

DROP TABLE pvactst2;

CREATE TABLE no_index_cleanup (i INT PRIMARY KEY, t TEXT);

CREATE INDEX no_index_cleanup_idx ON no_index_cleanup(t);

ALTER TABLE no_index_cleanup ALTER COLUMN t SET STORAGE EXTERNAL;

INSERT INTO no_index_cleanup(i, t) VALUES (generate_series(1,30),
    repeat('1234567890',269));

VACUUM (INDEX_CLEANUP TRUE, FULL TRUE) no_index_cleanup;

VACUUM (FULL TRUE) no_index_cleanup;

ALTER TABLE no_index_cleanup SET (vacuum_index_cleanup = false);

DELETE FROM no_index_cleanup WHERE i < 15;

VACUUM no_index_cleanup;

ALTER TABLE no_index_cleanup SET (vacuum_index_cleanup = true);

VACUUM no_index_cleanup;

ALTER TABLE no_index_cleanup SET (vacuum_index_cleanup = auto);

VACUUM no_index_cleanup;

INSERT INTO no_index_cleanup(i, t) VALUES (generate_series(31,60),
    repeat('1234567890',269));

DELETE FROM no_index_cleanup WHERE i < 45;

ALTER TABLE no_index_cleanup SET (vacuum_index_cleanup = off,
    toast.vacuum_index_cleanup = yes);

VACUUM no_index_cleanup;

ALTER TABLE no_index_cleanup SET (vacuum_index_cleanup = true,
    toast.vacuum_index_cleanup = false);

VACUUM no_index_cleanup;

VACUUM (INDEX_CLEANUP FALSE) vaccluster;

VACUUM (INDEX_CLEANUP AUTO) vactst;

VACUUM (INDEX_CLEANUP FALSE, FREEZE TRUE) vaccluster;

CREATE TEMP TABLE vac_truncate_test(i INT NOT NULL, j text)
	WITH (vacuum_truncate=true, autovacuum_enabled=false);

INSERT INTO vac_truncate_test VALUES (1, NULL), (NULL, NULL);

VACUUM (TRUNCATE FALSE, DISABLE_PAGE_SKIPPING) vac_truncate_test;

SELECT pg_relation_size('vac_truncate_test') > 0;

SET vacuum_truncate = false;

VACUUM (DISABLE_PAGE_SKIPPING) vac_truncate_test;

SELECT pg_relation_size('vac_truncate_test') = 0;

VACUUM (TRUNCATE FALSE, FULL TRUE) vac_truncate_test;

ALTER TABLE vac_truncate_test RESET (vacuum_truncate);

INSERT INTO vac_truncate_test VALUES (1, NULL), (NULL, NULL);

VACUUM (DISABLE_PAGE_SKIPPING) vac_truncate_test;

SELECT pg_relation_size('vac_truncate_test') > 0;

RESET vacuum_truncate;

VACUUM (TRUNCATE FALSE, DISABLE_PAGE_SKIPPING) vac_truncate_test;

SELECT pg_relation_size('vac_truncate_test') > 0;

VACUUM (DISABLE_PAGE_SKIPPING) vac_truncate_test;

SELECT pg_relation_size('vac_truncate_test') = 0;

DROP TABLE vac_truncate_test;

CREATE TABLE vacparted (a int, b char) PARTITION BY LIST (a);

CREATE TABLE vacparted1 PARTITION OF vacparted FOR VALUES IN (1);

INSERT INTO vacparted VALUES (1, 'a');

UPDATE vacparted SET b = 'b';

VACUUM (ANALYZE) vacparted;

VACUUM (FULL) vacparted;

VACUUM (FREEZE) vacparted;

VACUUM ANALYZE vacparted(a,b,a);

ANALYZE vacparted(a,b,b);

CREATE TABLE vacparted_i (a int primary key, b varchar(100))
  PARTITION BY HASH (a);

CREATE TABLE vacparted_i1 PARTITION OF vacparted_i
  FOR VALUES WITH (MODULUS 2, REMAINDER 0);

CREATE TABLE vacparted_i2 PARTITION OF vacparted_i
  FOR VALUES WITH (MODULUS 2, REMAINDER 1);

INSERT INTO vacparted_i SELECT i, 'test_'|| i from generate_series(1,10) i;

VACUUM (ANALYZE) vacparted_i;

VACUUM (FULL) vacparted_i;

VACUUM (FREEZE) vacparted_i;

SELECT relname, relhasindex FROM pg_class
  WHERE relname LIKE 'vacparted_i%' AND relkind IN ('p','r')
  ORDER BY relname;

DROP TABLE vacparted_i;

VACUUM vaccluster, vactst;

VACUUM vacparted, does_not_exist;

VACUUM (FREEZE) vacparted, vaccluster, vactst;

VACUUM (FREEZE) does_not_exist, vaccluster;

VACUUM ANALYZE vactst, vacparted (a);

VACUUM ANALYZE vactst (does_not_exist), vacparted (b);

VACUUM FULL vacparted, vactst;

VACUUM FULL vactst, vacparted (a, b), vaccluster (i);

ANALYZE vactst, vacparted;

ANALYZE vacparted (b), vactst;

ANALYZE vactst, does_not_exist, vacparted;

ANALYZE vactst (i), vacparted (does_not_exist);

ANALYZE vactst, vactst;

BEGIN;

ANALYZE vactst, vactst;

COMMIT;

CREATE TABLE only_parted (a int, b text) PARTITION BY LIST (a);

CREATE TABLE only_parted1 PARTITION OF only_parted FOR VALUES IN (1);

INSERT INTO only_parted VALUES (1, 'a');

SELECT relname, last_analyze IS NOT NULL AS analyzed, last_vacuum IS NOT NULL AS vacuumed
  FROM pg_stat_user_tables
  WHERE relid IN ('only_parted'::regclass, 'only_parted1'::regclass)
  ORDER BY relname;

ANALYZE only_parted;

SELECT relname, last_analyze IS NOT NULL AS analyzed, last_vacuum IS NOT NULL AS vacuumed
  FROM pg_stat_user_tables
  WHERE relid IN ('only_parted'::regclass, 'only_parted1'::regclass)
  ORDER BY relname;

DROP TABLE only_parted;

CREATE TABLE only_inh_parent (a int primary key, b TEXT);

CREATE TABLE only_inh_child () INHERITS (only_inh_parent);

INSERT INTO only_inh_child(a,b) VALUES (1, 'aaa'), (2, 'bbb'), (3, 'ccc');

SELECT relname, last_analyze IS NOT NULL AS analyzed, last_vacuum IS NOT NULL AS vacuumed
  FROM pg_stat_user_tables
  WHERE relid IN ('only_inh_parent'::regclass, 'only_inh_child'::regclass)
  ORDER BY relname;

ANALYZE only_inh_parent;

SELECT relname, last_analyze IS NOT NULL AS analyzed, last_vacuum IS NOT NULL AS vacuumed
  FROM pg_stat_user_tables
  WHERE relid IN ('only_inh_parent'::regclass, 'only_inh_child'::regclass)
  ORDER BY relname;

SELECT relname, last_analyze IS NOT NULL AS analyzed, last_vacuum IS NOT NULL AS vacuumed
  FROM pg_stat_user_tables
  WHERE relid IN ('only_inh_parent'::regclass, 'only_inh_child'::regclass)
  ORDER BY relname;

VACUUM only_inh_parent;

SELECT relname, last_analyze IS NOT NULL AS analyzed, last_vacuum IS NOT NULL AS vacuumed
  FROM pg_stat_user_tables
  WHERE relid IN ('only_inh_parent'::regclass, 'only_inh_child'::regclass)
  ORDER BY relname;

DROP TABLE only_inh_parent CASCADE;

ANALYZE (VERBOSE) does_not_exist;

ANALYZE (nonexistentarg) does_not_exit;

SET client_min_messages TO 'ERROR';

ANALYZE (SKIP_LOCKED, VERBOSE) does_not_exist;

ANALYZE (VERBOSE, SKIP_LOCKED) does_not_exist;

VACUUM (SKIP_LOCKED) vactst;

VACUUM (SKIP_LOCKED, FULL) vactst;

ANALYZE (SKIP_LOCKED) vactst;

RESET client_min_messages;

SET default_transaction_isolation = serializable;

VACUUM vactst;

ANALYZE vactst;

RESET default_transaction_isolation;

BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE;

ANALYZE vactst;

COMMIT;

CREATE TABLE vac_option_tab (a INT, t TEXT);

INSERT INTO vac_option_tab SELECT a, 't' || a FROM generate_series(1, 10) AS a;

ALTER TABLE vac_option_tab ALTER COLUMN t SET STORAGE EXTERNAL;

CREATE VIEW vac_option_tab_counts AS
  SELECT CASE WHEN c.relname IS NULL
    THEN 'main' ELSE 'toast' END as rel,
  s.vacuum_count
  FROM pg_stat_all_tables s
  LEFT JOIN pg_class c ON s.relid = c.reltoastrelid
  WHERE c.relname = 'vac_option_tab' OR s.relname = 'vac_option_tab'
  ORDER BY rel;

VACUUM (PROCESS_TOAST TRUE) vac_option_tab;

SELECT * FROM vac_option_tab_counts;

VACUUM (PROCESS_TOAST FALSE) vac_option_tab;

SELECT * FROM vac_option_tab_counts;

VACUUM (PROCESS_TOAST FALSE, FULL) vac_option_tab;

VACUUM (PROCESS_MAIN FALSE) vac_option_tab;

SELECT * FROM vac_option_tab_counts;

VACUUM (PROCESS_MAIN FALSE, PROCESS_TOAST FALSE) vac_option_tab;

SELECT * FROM vac_option_tab_counts;

SELECT relfilenode AS main_filenode FROM pg_class
  WHERE relname = 'vac_option_tab' ;

SELECT t.relfilenode AS toast_filenode FROM pg_class c, pg_class t
  WHERE c.reltoastrelid = t.oid AND c.relname = 'vac_option_tab' ;

VACUUM (PROCESS_MAIN FALSE, FULL) vac_option_tab;

SELECT relfilenode = 'main_filenode' AS is_same_main_filenode
  FROM pg_class WHERE relname = 'vac_option_tab';

SELECT t.relfilenode = 'toast_filenode' AS is_same_toast_filenode
  FROM pg_class c, pg_class t
  WHERE c.reltoastrelid = t.oid AND c.relname = 'vac_option_tab';

VACUUM (BUFFER_USAGE_LIMIT '512 kB') vac_option_tab;

ANALYZE (BUFFER_USAGE_LIMIT '512 kB') vac_option_tab;

VACUUM (BUFFER_USAGE_LIMIT 0) vac_option_tab;

ANALYZE (BUFFER_USAGE_LIMIT 0) vac_option_tab;

VACUUM (BUFFER_USAGE_LIMIT 16777220) vac_option_tab;

VACUUM (BUFFER_USAGE_LIMIT 120) vac_option_tab;

VACUUM (BUFFER_USAGE_LIMIT 10000000000) vac_option_tab;

VACUUM (BUFFER_USAGE_LIMIT '512 kB', FULL) vac_option_tab;

VACUUM (SKIP_DATABASE_STATS) vactst;

VACUUM (ONLY_DATABASE_STATS);

VACUUM (ONLY_DATABASE_STATS) vactst;

DROP VIEW vac_option_tab_counts;

DROP TABLE vac_option_tab;

DROP TABLE vaccluster;

DROP TABLE vactst;

DROP TABLE vacparted;

DROP TABLE no_index_cleanup;

CREATE TABLE vacowned (a int);

CREATE TABLE vacowned_parted (a int) PARTITION BY LIST (a);

CREATE TABLE vacowned_part1 PARTITION OF vacowned_parted FOR VALUES IN (1);

CREATE TABLE vacowned_part2 PARTITION OF vacowned_parted FOR VALUES IN (2);

CREATE ROLE regress_vacuum;

SET ROLE regress_vacuum;

VACUUM vacowned;

ANALYZE vacowned;

VACUUM (ANALYZE) vacowned;

VACUUM pg_catalog.pg_class;

ANALYZE pg_catalog.pg_class;

VACUUM (ANALYZE) pg_catalog.pg_class;

VACUUM pg_catalog.pg_authid;

ANALYZE pg_catalog.pg_authid;

VACUUM (ANALYZE) pg_catalog.pg_authid;

VACUUM vacowned_parted;

VACUUM vacowned_part1;

VACUUM vacowned_part2;

ANALYZE vacowned_parted;

ANALYZE vacowned_part1;

ANALYZE vacowned_part2;

VACUUM (ANALYZE) vacowned_parted;

VACUUM (ANALYZE) vacowned_part1;

VACUUM (ANALYZE) vacowned_part2;

RESET ROLE;

ALTER TABLE vacowned_parted OWNER TO regress_vacuum;

ALTER TABLE vacowned_part1 OWNER TO regress_vacuum;

SET ROLE regress_vacuum;

VACUUM vacowned_parted;

VACUUM vacowned_part1;

VACUUM vacowned_part2;

ANALYZE vacowned_parted;

ANALYZE vacowned_part1;

ANALYZE vacowned_part2;

VACUUM (ANALYZE) vacowned_parted;

VACUUM (ANALYZE) vacowned_part1;

VACUUM (ANALYZE) vacowned_part2;

RESET ROLE;

ALTER TABLE vacowned_parted OWNER TO CURRENT_USER;

SET ROLE regress_vacuum;

VACUUM vacowned_parted;

VACUUM vacowned_part1;

VACUUM vacowned_part2;

ANALYZE vacowned_parted;

ANALYZE vacowned_part1;

ANALYZE vacowned_part2;

VACUUM (ANALYZE) vacowned_parted;

VACUUM (ANALYZE) vacowned_part1;

VACUUM (ANALYZE) vacowned_part2;

RESET ROLE;

ALTER TABLE vacowned_parted OWNER TO regress_vacuum;

ALTER TABLE vacowned_part1 OWNER TO CURRENT_USER;

SET ROLE regress_vacuum;

VACUUM vacowned_parted;

VACUUM vacowned_part1;

VACUUM vacowned_part2;

ANALYZE vacowned_parted;

ANALYZE vacowned_part1;

ANALYZE vacowned_part2;

VACUUM (ANALYZE) vacowned_parted;

VACUUM (ANALYZE) vacowned_part1;

VACUUM (ANALYZE) vacowned_part2;

RESET ROLE;

DROP TABLE vacowned;

DROP TABLE vacowned_parted;

DROP ROLE regress_vacuum;

CREATE TABLE vac_rewrite_toast (id int, f1 TEXT STORAGE plain);

INSERT INTO vac_rewrite_toast values (1, repeat('a', 7000));

ALTER TABLE vac_rewrite_toast ALTER COLUMN f1 SET STORAGE EXTERNAL;

INSERT INTO vac_rewrite_toast values (2, repeat('a', 7000));

SELECT pg_column_toast_chunk_id(f1) AS id_2_chunk FROM vac_rewrite_toast
  WHERE id = 2 ;

SELECT id, pg_column_toast_chunk_id(f1) IS NULL AS f1_chunk_null,
  substr(f1, 5, 10) AS f1_data,
  pg_column_compression(f1) AS f1_comp
  FROM vac_rewrite_toast ORDER BY id;

VACUUM FULL vac_rewrite_toast;

SELECT id, pg_column_toast_chunk_id(f1) IS NULL AS f1_chunk_null,
  substr(f1, 5, 10) AS f1_data,
  pg_column_compression(f1) AS f1_comp
  FROM vac_rewrite_toast ORDER BY id;

SELECT pg_column_toast_chunk_id(f1) = 'id_2_chunk' AS same_chunk
  FROM vac_rewrite_toast WHERE id = 2;

DROP TABLE vac_rewrite_toast;
