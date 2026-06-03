-- Regression coverage: identifiers that require quoting (reserved keywords like
-- "primary", or multi-word/mixed-case names) must keep their quotes through the
-- formatter. Each statement exercises a different emitter that previously emitted
-- the name as a bare token, producing invalid SQL.

-- tables / columns / constraints / schema
CREATE TABLE "primary" (x int);
CREATE TABLE t1 ("primary" int);
CREATE TABLE t2 (x int CONSTRAINT "primary" CHECK (x > 0));
ALTER TABLE t1 SET SCHEMA "primary";

-- view / sequence / domain / collation
CREATE VIEW "primary" AS SELECT 1;
CREATE SEQUENCE "primary";
CREATE DOMAIN "primary" AS int;
CREATE COLLATION "primary" FROM "default";

-- index (index name, indexed column, tablespace)
CREATE INDEX "primary" ON t1 (x);
CREATE UNIQUE INDEX i1 ON t1 USING btree (x, "primary") TABLESPACE "primary" WHERE "primary";
CLUSTER t1 USING "primary";

-- CTE / table alias / column alias
WITH "primary" AS (SELECT 1 AS a) SELECT * FROM "primary";
SELECT * FROM t1 AS "primary";
SELECT * FROM t1 "Mixed Case";
SELECT * FROM (SELECT 1) AS x("primary");

-- trigger / event trigger / rule
CREATE TRIGGER "primary" BEFORE INSERT ON t1 FOR EACH ROW EXECUTE FUNCTION f();
CREATE EVENT TRIGGER "primary" ON ddl_command_start EXECUTE FUNCTION f();
ALTER EVENT TRIGGER "primary" ENABLE;
CREATE RULE "primary" AS ON INSERT TO t1 DO NOTHING;

-- prepared statements / cursors / notifications
PREPARE "primary" AS SELECT 1;
EXECUTE "primary";
DEALLOCATE "primary";
LISTEN "primary";
UNLISTEN "primary";
NOTIFY "primary";
CLOSE "primary";
FETCH 1 FROM "primary";

-- extensions / publications
CREATE EXTENSION "primary";
ALTER EXTENSION "primary" ADD TABLE t1;
CREATE PUBLICATION "primary";
ALTER PUBLICATION "primary" ADD TABLE t1;

-- foreign data wrappers / servers / user mappings / foreign tables
CREATE FOREIGN DATA WRAPPER "primary";
CREATE USER MAPPING FOR u SERVER "primary";
DROP USER MAPPING FOR u SERVER "primary";
CREATE FOREIGN TABLE ft1 (x int) SERVER "primary";
IMPORT FOREIGN SCHEMA "primary" FROM SERVER s INTO "select";

-- databases / roles / tablespaces
DROP DATABASE "primary";
ALTER DATABASE "primary" SET work_mem = '1MB';
ALTER ROLE r IN DATABASE "primary" SET work_mem = '1MB';
DROP TABLESPACE "primary";
ALTER TABLE ALL IN TABLESPACE "primary" SET TABLESPACE "select";

-- access methods / operator classes & families / exclusion constraints
CREATE INDEX i2 ON t1 USING "primary" (x);
CREATE ACCESS METHOD "primary" TYPE INDEX HANDLER h;
CREATE OPERATOR FAMILY of1 USING "primary";
ALTER OPERATOR FAMILY of1 USING "primary" ADD OPERATOR 1 = (int, int);
CREATE OPERATOR CLASS oc1 FOR TYPE int USING "primary" AS OPERATOR 1 =;
ALTER TABLE t1 ADD CONSTRAINT e1 EXCLUDE USING "primary" (x WITH =);

-- procedural languages / transforms / partitioning / replica identity / reindex
CREATE TRANSFORM FOR int LANGUAGE "primary" (FROM SQL WITH FUNCTION f(internal), TO SQL WITH FUNCTION g(int));
CREATE TABLE t3 (x int) PARTITION BY LIST ("primary");
ALTER TABLE t1 REPLICA IDENTITY USING INDEX "primary";
REINDEX SCHEMA "primary";
ALTER FUNCTION f() DEPENDS ON EXTENSION "primary";
ALTER PUBLICATION p1 ADD TABLES IN SCHEMA "primary";

-- mixed-case identifiers (must keep quotes to preserve case)
REINDEX SCHEMA "MySchema";
SELECT * FROM t1 AS "MyAlias" ("MyCol");

ALTER TABLE t1 DROP COLUMN "primary";
ALTER TABLE t1 ALTER COLUMN "primary" SET NOT NULL;
ALTER DOMAIN dom DROP CONSTRAINT "primary";
ALTER TABLESPACE "primary" SET (seq_page_cost = 1.0);
CREATE TABLE t1 (c text COMPRESSION "primary");
CREATE PUBLICATION p1 FOR TABLES IN SCHEMA "primary";
CREATE TABLE t1 (x int) TABLESPACE "primary";
CREATE TABLE t1 USING "primary" AS SELECT 1 AS a;
CREATE TABLE t2 TABLESPACE "primary" AS SELECT 1 AS a;
CREATE TRIGGER tr1 AFTER UPDATE ON t1 REFERENCING OLD TABLE AS "primary" FOR EACH STATEMENT EXECUTE FUNCTION f();
UPDATE t1 SET x = 1 WHERE CURRENT OF "primary";
DROP SUBSCRIPTION "primary";
