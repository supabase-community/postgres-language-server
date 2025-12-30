CREATE TABLE reloptions_test(i INT) WITH (FiLLFaCToR=30,
	autovacuum_enabled = false, autovacuum_analyze_scale_factor = 0.2);

SELECT reloptions FROM pg_class WHERE oid = 'reloptions_test'::regclass;

CREATE TABLE reloptions_test2(i INT) WITH (fillfactor=2);

CREATE TABLE reloptions_test2(i INT) WITH (fillfactor=110);

CREATE TABLE reloptions_test2(i INT) WITH (autovacuum_analyze_scale_factor = -10.0);

CREATE TABLE reloptions_test2(i INT) WITH (autovacuum_analyze_scale_factor = 110.0);

CREATE TABLE reloptions_test2(i INT) WITH (not_existing_option=2);

CREATE TABLE reloptions_test2(i INT) WITH (not_existing_namespace.fillfactor=2);

CREATE TABLE reloptions_test2(i INT) WITH (fillfactor=-30.1);

CREATE TABLE reloptions_test2(i INT) WITH (fillfactor='string');

CREATE TABLE reloptions_test2(i INT) WITH (fillfactor=true);

CREATE TABLE reloptions_test2(i INT) WITH (autovacuum_enabled=12);

CREATE TABLE reloptions_test2(i INT) WITH (autovacuum_enabled=30.5);

CREATE TABLE reloptions_test2(i INT) WITH (autovacuum_enabled='string');

CREATE TABLE reloptions_test2(i INT) WITH (autovacuum_analyze_scale_factor='string');

CREATE TABLE reloptions_test2(i INT) WITH (autovacuum_analyze_scale_factor=true);

CREATE TABLE reloptions_test2(i INT) WITH (fillfactor=30, fillfactor=40);

CREATE TABLE reloptions_test2(i INT) WITH (fillfactor);

ALTER TABLE reloptions_test SET (fillfactor=31,
	autovacuum_analyze_scale_factor = 0.3);

SELECT reloptions FROM pg_class WHERE oid = 'reloptions_test'::regclass;

ALTER TABLE reloptions_test SET (autovacuum_enabled, fillfactor=32);

SELECT reloptions FROM pg_class WHERE oid = 'reloptions_test'::regclass;

ALTER TABLE reloptions_test RESET (fillfactor);

SELECT reloptions FROM pg_class WHERE oid = 'reloptions_test'::regclass;

ALTER TABLE reloptions_test RESET (autovacuum_enabled,
	autovacuum_analyze_scale_factor);

SELECT reloptions FROM pg_class WHERE oid = 'reloptions_test'::regclass AND
       reloptions IS NULL;

ALTER TABLE reloptions_test RESET (fillfactor=12);

UPDATE pg_class
	SET reloptions = '{fillfactor=13,autovacuum_enabled=false,illegal_option=4}'
	WHERE oid = 'reloptions_test'::regclass;

ALTER TABLE reloptions_test RESET (illegal_option);

SELECT reloptions FROM pg_class WHERE oid = 'reloptions_test'::regclass;

DROP TABLE reloptions_test;

CREATE TEMP TABLE reloptions_test(i INT NOT NULL, j text)
	WITH (vacuum_truncate=false,
	toast.vacuum_truncate=false,
	autovacuum_enabled=false);

SELECT reloptions FROM pg_class WHERE oid = 'reloptions_test'::regclass;

INSERT INTO reloptions_test VALUES (1, NULL), (NULL, NULL);

VACUUM (FREEZE, DISABLE_PAGE_SKIPPING) reloptions_test;

SELECT pg_relation_size('reloptions_test') > 0;

SELECT reloptions FROM pg_class WHERE oid =
	(SELECT reltoastrelid FROM pg_class
	WHERE oid = 'reloptions_test'::regclass);

ALTER TABLE reloptions_test RESET (vacuum_truncate);

SELECT reloptions FROM pg_class WHERE oid = 'reloptions_test'::regclass;

INSERT INTO reloptions_test VALUES (1, NULL), (NULL, NULL);

VACUUM (FREEZE, DISABLE_PAGE_SKIPPING) reloptions_test;

SELECT pg_relation_size('reloptions_test') = 0;

DROP TABLE reloptions_test;

CREATE TABLE reloptions_test (s VARCHAR)
	WITH (toast.autovacuum_vacuum_cost_delay = 23);

SELECT reltoastrelid as toast_oid
	FROM pg_class WHERE oid = 'reloptions_test'::regclass ;

SELECT reloptions FROM pg_class WHERE oid = 'toast_oid';

ALTER TABLE reloptions_test SET (toast.autovacuum_vacuum_cost_delay = 24);

SELECT reloptions FROM pg_class WHERE oid = 'toast_oid';

ALTER TABLE reloptions_test RESET (toast.autovacuum_vacuum_cost_delay);

SELECT reloptions FROM pg_class WHERE oid = 'toast_oid';

CREATE TABLE reloptions_test2 (i int) WITH (toast.not_existing_option = 42);

DROP TABLE reloptions_test;

CREATE TABLE reloptions_test (s VARCHAR) WITH
	(toast.autovacuum_vacuum_cost_delay = 23,
	autovacuum_vacuum_cost_delay = 24, fillfactor = 40);

SELECT reloptions FROM pg_class WHERE oid = 'reloptions_test'::regclass;

SELECT reloptions FROM pg_class WHERE oid = (
	SELECT reltoastrelid FROM pg_class WHERE oid = 'reloptions_test'::regclass);

CREATE INDEX reloptions_test_idx ON reloptions_test (s) WITH (fillfactor=30);

SELECT reloptions FROM pg_class WHERE oid = 'reloptions_test_idx'::regclass;

CREATE INDEX reloptions_test_idx ON reloptions_test (s)
	WITH (not_existing_option=2);

CREATE INDEX reloptions_test_idx ON reloptions_test (s)
	WITH (not_existing_ns.fillfactor=2);

CREATE INDEX reloptions_test_idx2 ON reloptions_test (s) WITH (fillfactor=1);

CREATE INDEX reloptions_test_idx2 ON reloptions_test (s) WITH (fillfactor=130);

ALTER INDEX reloptions_test_idx SET (fillfactor=40);

SELECT reloptions FROM pg_class WHERE oid = 'reloptions_test_idx'::regclass;

CREATE INDEX reloptions_test_idx3 ON reloptions_test (s);

ALTER INDEX reloptions_test_idx3 SET (fillfactor=40);

SELECT reloptions FROM pg_class WHERE oid = 'reloptions_test_idx3'::regclass;
