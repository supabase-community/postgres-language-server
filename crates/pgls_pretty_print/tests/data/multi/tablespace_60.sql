CREATE TABLESPACE regress_tblspace LOCATION 'relative';

CREATE TABLESPACE regress_tblspace LOCATION '';

SET allow_in_place_tablespaces = true;

CREATE TABLESPACE regress_tblspacewith LOCATION '' WITH (some_nonexistent_parameter = true);

CREATE TABLESPACE regress_tblspacewith LOCATION '' WITH (random_page_cost = 3.0);

SELECT spcoptions FROM pg_tablespace WHERE spcname = 'regress_tblspacewith';

DROP TABLESPACE regress_tblspacewith;

SELECT regexp_replace(pg_tablespace_location(oid), '(pg_tblspc)/(\d+)', '\1/NNN')
  FROM pg_tablespace  WHERE spcname = 'regress_tblspace';

ALTER TABLESPACE regress_tblspace SET (random_page_cost = 1.0, seq_page_cost = 1.1);

ALTER TABLESPACE regress_tblspace SET (some_nonexistent_parameter = true);

ALTER TABLESPACE regress_tblspace RESET (random_page_cost = 2.0);

ALTER TABLESPACE regress_tblspace RESET (random_page_cost, effective_io_concurrency);

REINDEX (TABLESPACE regress_tblspace) TABLE pg_am;

REINDEX (TABLESPACE regress_tblspace) TABLE CONCURRENTLY pg_am;

REINDEX (TABLESPACE regress_tblspace) TABLE pg_authid;

REINDEX (TABLESPACE regress_tblspace) TABLE CONCURRENTLY pg_authid;

REINDEX (TABLESPACE regress_tblspace) INDEX pg_toast.pg_toast_1262_index;

REINDEX (TABLESPACE regress_tblspace) INDEX CONCURRENTLY pg_toast.pg_toast_1262_index;

REINDEX (TABLESPACE regress_tblspace) TABLE pg_toast.pg_toast_1262;

REINDEX (TABLESPACE regress_tblspace) TABLE CONCURRENTLY pg_toast.pg_toast_1262;

REINDEX (TABLESPACE pg_global) TABLE pg_authid;

REINDEX (TABLESPACE pg_global) TABLE CONCURRENTLY pg_authid;

CREATE TABLE regress_tblspace_test_tbl (num1 bigint, num2 double precision, t text);

INSERT INTO regress_tblspace_test_tbl (num1, num2, t)
  SELECT round(random()*100), random(), 'text'
  FROM generate_series(1, 10) s(i);

CREATE INDEX regress_tblspace_test_tbl_idx ON regress_tblspace_test_tbl (num1);

REINDEX (TABLESPACE pg_global) INDEX regress_tblspace_test_tbl_idx;

REINDEX (TABLESPACE pg_global) INDEX CONCURRENTLY regress_tblspace_test_tbl_idx;

BEGIN;

REINDEX (TABLESPACE regress_tblspace) INDEX regress_tblspace_test_tbl_idx;

REINDEX (TABLESPACE regress_tblspace) TABLE regress_tblspace_test_tbl;

ROLLBACK;

SELECT c.relname FROM pg_class c, pg_tablespace s
  WHERE c.reltablespace = s.oid AND s.spcname = 'regress_tblspace';

SELECT relfilenode as main_filenode FROM pg_class
  WHERE relname = 'regress_tblspace_test_tbl_idx' ;

SELECT relfilenode as toast_filenode FROM pg_class
  WHERE oid =
    (SELECT i.indexrelid
       FROM pg_class c,
            pg_index i
       WHERE i.indrelid = c.reltoastrelid AND
             c.relname = 'regress_tblspace_test_tbl') ;

REINDEX (TABLESPACE regress_tblspace) TABLE regress_tblspace_test_tbl;

SELECT c.relname FROM pg_class c, pg_tablespace s
  WHERE c.reltablespace = s.oid AND s.spcname = 'regress_tblspace'
  ORDER BY c.relname;

ALTER TABLE regress_tblspace_test_tbl SET TABLESPACE regress_tblspace;

ALTER TABLE regress_tblspace_test_tbl SET TABLESPACE pg_default;

SELECT c.relname FROM pg_class c, pg_tablespace s
  WHERE c.reltablespace = s.oid AND s.spcname = 'regress_tblspace'
  ORDER BY c.relname;

ALTER INDEX regress_tblspace_test_tbl_idx SET TABLESPACE pg_default;

SELECT c.relname FROM pg_class c, pg_tablespace s
  WHERE c.reltablespace = s.oid AND s.spcname = 'regress_tblspace'
  ORDER BY c.relname;

REINDEX (TABLESPACE regress_tblspace, CONCURRENTLY) TABLE regress_tblspace_test_tbl;

SELECT c.relname FROM pg_class c, pg_tablespace s
  WHERE c.reltablespace = s.oid AND s.spcname = 'regress_tblspace'
  ORDER BY c.relname;

SELECT relfilenode = 'main_filenode' AS main_same FROM pg_class
  WHERE relname = 'regress_tblspace_test_tbl_idx';

SELECT relfilenode = 'toast_filenode' as toast_same FROM pg_class
  WHERE oid =
    (SELECT i.indexrelid
       FROM pg_class c,
            pg_index i
       WHERE i.indrelid = c.reltoastrelid AND
             c.relname = 'regress_tblspace_test_tbl');

DROP TABLE regress_tblspace_test_tbl;

CREATE TABLE tbspace_reindex_part (c1 int, c2 int) PARTITION BY RANGE (c1);

CREATE TABLE tbspace_reindex_part_0 PARTITION OF tbspace_reindex_part
  FOR VALUES FROM (0) TO (10) PARTITION BY list (c2);

CREATE TABLE tbspace_reindex_part_0_1 PARTITION OF tbspace_reindex_part_0
  FOR VALUES IN (1);

CREATE TABLE tbspace_reindex_part_0_2 PARTITION OF tbspace_reindex_part_0
  FOR VALUES IN (2);

CREATE TABLE tbspace_reindex_part_10 PARTITION OF tbspace_reindex_part
   FOR VALUES FROM (10) TO (20) PARTITION BY list (c2);

CREATE INDEX tbspace_reindex_part_index ON ONLY tbspace_reindex_part (c1);

CREATE INDEX tbspace_reindex_part_index_0 ON ONLY tbspace_reindex_part_0 (c1);

ALTER INDEX tbspace_reindex_part_index ATTACH PARTITION tbspace_reindex_part_index_0;

CREATE INDEX tbspace_reindex_part_index_10 ON ONLY tbspace_reindex_part_10 (c1);

ALTER INDEX tbspace_reindex_part_index ATTACH PARTITION tbspace_reindex_part_index_10;

CREATE INDEX tbspace_reindex_part_index_0_1 ON ONLY tbspace_reindex_part_0_1 (c1);

ALTER INDEX tbspace_reindex_part_index_0 ATTACH PARTITION tbspace_reindex_part_index_0_1;

CREATE INDEX tbspace_reindex_part_index_0_2 ON ONLY tbspace_reindex_part_0_2 (c1);

ALTER INDEX tbspace_reindex_part_index_0 ATTACH PARTITION tbspace_reindex_part_index_0_2;

SELECT relid, parentrelid, level FROM pg_partition_tree('tbspace_reindex_part_index')
  ORDER BY relid, level;

CREATE TEMP TABLE reindex_temp_before AS
  SELECT oid, relname, relfilenode, reltablespace
  FROM pg_class
    WHERE relname ~ 'tbspace_reindex_part_index';

REINDEX (TABLESPACE regress_tblspace, CONCURRENTLY) TABLE tbspace_reindex_part;

SELECT b.relname,
       CASE WHEN a.relfilenode = b.relfilenode THEN 'relfilenode is unchanged'
       ELSE 'relfilenode has changed' END AS filenode,
       CASE WHEN a.reltablespace = b.reltablespace THEN 'reltablespace is unchanged'
       ELSE 'reltablespace has changed' END AS tbspace
  FROM reindex_temp_before b JOIN pg_class a ON b.relname = a.relname
  ORDER BY 1;

DROP TABLE tbspace_reindex_part;

CREATE SCHEMA testschema;

CREATE TABLE testschema.foo (i int) TABLESPACE regress_tblspace;

SELECT relname, spcname FROM pg_catalog.pg_tablespace t, pg_catalog.pg_class c
    where c.reltablespace = t.oid AND c.relname = 'foo';

INSERT INTO testschema.foo VALUES(1);

INSERT INTO testschema.foo VALUES(2);

CREATE TABLE testschema.asselect TABLESPACE regress_tblspace AS SELECT 1;

SELECT relname, spcname FROM pg_catalog.pg_tablespace t, pg_catalog.pg_class c
    where c.reltablespace = t.oid AND c.relname = 'asselect';

PREPARE selectsource(int) AS SELECT $1;

CREATE TABLE testschema.asexecute TABLESPACE regress_tblspace
    AS EXECUTE selectsource(2);

SELECT relname, spcname FROM pg_catalog.pg_tablespace t, pg_catalog.pg_class c
    where c.reltablespace = t.oid AND c.relname = 'asexecute';

CREATE INDEX foo_idx on testschema.foo(i) TABLESPACE regress_tblspace;

SELECT relname, spcname FROM pg_catalog.pg_tablespace t, pg_catalog.pg_class c
    where c.reltablespace = t.oid AND c.relname = 'foo_idx';

CREATE TABLE testschema.part (a int) PARTITION BY LIST (a);

SET default_tablespace TO pg_global;

CREATE TABLE testschema.part_1 PARTITION OF testschema.part FOR VALUES IN (1);

RESET default_tablespace;

CREATE TABLE testschema.part_1 PARTITION OF testschema.part FOR VALUES IN (1);

SET default_tablespace TO regress_tblspace;

CREATE TABLE testschema.part_2 PARTITION OF testschema.part FOR VALUES IN (2);

SET default_tablespace TO pg_global;

CREATE TABLE testschema.part_3 PARTITION OF testschema.part FOR VALUES IN (3);

ALTER TABLE testschema.part SET TABLESPACE regress_tblspace;

CREATE TABLE testschema.part_3 PARTITION OF testschema.part FOR VALUES IN (3);

CREATE TABLE testschema.part_4 PARTITION OF testschema.part FOR VALUES IN (4)
  TABLESPACE pg_default;

CREATE TABLE testschema.part_56 PARTITION OF testschema.part FOR VALUES IN (5, 6)
  PARTITION BY LIST (a);

ALTER TABLE testschema.part SET TABLESPACE pg_default;

CREATE TABLE testschema.part_78 PARTITION OF testschema.part FOR VALUES IN (7, 8)
  PARTITION BY LIST (a);

CREATE TABLE testschema.part_910 PARTITION OF testschema.part FOR VALUES IN (9, 10)
  PARTITION BY LIST (a) TABLESPACE regress_tblspace;

RESET default_tablespace;

CREATE TABLE testschema.part_78 PARTITION OF testschema.part FOR VALUES IN (7, 8)
  PARTITION BY LIST (a);

SELECT relname, spcname FROM pg_catalog.pg_class c
    JOIN pg_catalog.pg_namespace n ON (c.relnamespace = n.oid)
    LEFT JOIN pg_catalog.pg_tablespace t ON c.reltablespace = t.oid
    where c.relname LIKE 'part%' AND n.nspname = 'testschema' order by relname;

RESET default_tablespace;

DROP TABLE testschema.part;

CREATE TABLE testschema.part (a int) PARTITION BY LIST (a);

CREATE TABLE testschema.part1 PARTITION OF testschema.part FOR VALUES IN (1);

CREATE INDEX part_a_idx ON testschema.part (a) TABLESPACE regress_tblspace;

CREATE TABLE testschema.part2 PARTITION OF testschema.part FOR VALUES IN (2);

SELECT relname, spcname FROM pg_catalog.pg_tablespace t, pg_catalog.pg_class c
    where c.reltablespace = t.oid AND c.relname LIKE 'part%_idx' ORDER BY relname;

CREATE TABLE testschema.dflt (a int PRIMARY KEY) PARTITION BY LIST (a) TABLESPACE pg_default;

CREATE TABLE testschema.dflt (a int PRIMARY KEY USING INDEX TABLESPACE pg_default) PARTITION BY LIST (a);

SET default_tablespace TO 'pg_default';

CREATE TABLE testschema.dflt (a int PRIMARY KEY) PARTITION BY LIST (a) TABLESPACE regress_tblspace;

CREATE TABLE testschema.dflt (a int PRIMARY KEY USING INDEX TABLESPACE regress_tblspace) PARTITION BY LIST (a);

CREATE TABLE testschema.dflt (a int PRIMARY KEY USING INDEX TABLESPACE regress_tblspace) PARTITION BY LIST (a) TABLESPACE regress_tblspace;

SET default_tablespace TO '';

CREATE TABLE testschema.dflt2 (a int PRIMARY KEY) PARTITION BY LIST (a);

DROP TABLE testschema.dflt, testschema.dflt2;

CREATE TABLE testschema.test_default_tab(id bigint) TABLESPACE regress_tblspace;

INSERT INTO testschema.test_default_tab VALUES (1);

CREATE INDEX test_index1 on testschema.test_default_tab (id);

CREATE INDEX test_index2 on testschema.test_default_tab (id) TABLESPACE regress_tblspace;

ALTER TABLE testschema.test_default_tab ADD CONSTRAINT test_index3 PRIMARY KEY (id);

ALTER TABLE testschema.test_default_tab ADD CONSTRAINT test_index4 UNIQUE (id) USING INDEX TABLESPACE regress_tblspace;

SET default_tablespace TO regress_tblspace;

ALTER TABLE testschema.test_default_tab ALTER id TYPE bigint;

SELECT * FROM testschema.test_default_tab;

ALTER TABLE testschema.test_default_tab ALTER id TYPE int;

SELECT * FROM testschema.test_default_tab;

SET default_tablespace TO '';

ALTER TABLE testschema.test_default_tab ALTER id TYPE int;

ALTER TABLE testschema.test_default_tab ALTER id TYPE bigint;

DROP TABLE testschema.test_default_tab;

CREATE TABLE testschema.test_default_tab_p(id bigint, val bigint)
    PARTITION BY LIST (id) TABLESPACE regress_tblspace;

CREATE TABLE testschema.test_default_tab_p1 PARTITION OF testschema.test_default_tab_p
    FOR VALUES IN (1);

INSERT INTO testschema.test_default_tab_p VALUES (1);

CREATE INDEX test_index1 on testschema.test_default_tab_p (val);

CREATE INDEX test_index2 on testschema.test_default_tab_p (val) TABLESPACE regress_tblspace;

ALTER TABLE testschema.test_default_tab_p ADD CONSTRAINT test_index3 PRIMARY KEY (id);

ALTER TABLE testschema.test_default_tab_p ADD CONSTRAINT test_index4 UNIQUE (id) USING INDEX TABLESPACE regress_tblspace;

SET default_tablespace TO regress_tblspace;

ALTER TABLE testschema.test_default_tab_p ALTER val TYPE bigint;

SELECT * FROM testschema.test_default_tab_p;

ALTER TABLE testschema.test_default_tab_p ALTER val TYPE int;

SELECT * FROM testschema.test_default_tab_p;

SET default_tablespace TO '';

ALTER TABLE testschema.test_default_tab_p ALTER val TYPE int;

ALTER TABLE testschema.test_default_tab_p ALTER val TYPE bigint;

DROP TABLE testschema.test_default_tab_p;

CREATE TABLE testschema.test_tab(id int) TABLESPACE regress_tblspace;

INSERT INTO testschema.test_tab VALUES (1);

SET default_tablespace TO regress_tblspace;

ALTER TABLE testschema.test_tab ADD CONSTRAINT test_tab_unique UNIQUE (id);

SET default_tablespace TO '';

ALTER TABLE testschema.test_tab ADD CONSTRAINT test_tab_pkey PRIMARY KEY (id);

SELECT * FROM testschema.test_tab;

DROP TABLE testschema.test_tab;

CREATE TABLE testschema.test_tab(a int, b int, c int);

SET default_tablespace TO regress_tblspace;

ALTER TABLE testschema.test_tab ADD CONSTRAINT test_tab_unique UNIQUE (a);

CREATE INDEX test_tab_a_idx ON testschema.test_tab (a);

SET default_tablespace TO '';

CREATE INDEX test_tab_b_idx ON testschema.test_tab (b);

ALTER TABLE testschema.test_tab ALTER b TYPE bigint, ADD UNIQUE (c);

DROP TABLE testschema.test_tab;

CREATE TABLE testschema.atable AS VALUES (1), (2);

CREATE UNIQUE INDEX anindex ON testschema.atable(column1);

ALTER TABLE testschema.atable SET TABLESPACE regress_tblspace;

ALTER INDEX testschema.anindex SET TABLESPACE regress_tblspace;

ALTER INDEX testschema.part_a_idx SET TABLESPACE pg_global;

ALTER INDEX testschema.part_a_idx SET TABLESPACE pg_default;

ALTER INDEX testschema.part_a_idx SET TABLESPACE regress_tblspace;

INSERT INTO testschema.atable VALUES(3);

INSERT INTO testschema.atable VALUES(1);

SELECT COUNT(*) FROM testschema.atable;

CREATE MATERIALIZED VIEW testschema.amv AS SELECT * FROM testschema.atable;

ALTER MATERIALIZED VIEW testschema.amv SET TABLESPACE regress_tblspace;

REFRESH MATERIALIZED VIEW testschema.amv;

SELECT COUNT(*) FROM testschema.amv;

CREATE TABLESPACE regress_badspace LOCATION '/no/such/location';

CREATE TABLE bar (i int) TABLESPACE regress_nosuchspace;

DROP TABLESPACE regress_tblspace;

ALTER INDEX testschema.part_a_idx SET TABLESPACE pg_default;

DROP TABLESPACE regress_tblspace;

BEGIN;

GRANT ALL ON TABLESPACE regress_tblspace TO PUBLIC;

ROLLBACK;

CREATE ROLE regress_tablespace_user1 login;

CREATE ROLE regress_tablespace_user2 login;

GRANT USAGE ON SCHEMA testschema TO regress_tablespace_user2;

ALTER TABLESPACE regress_tblspace OWNER TO regress_tablespace_user1;

CREATE TABLE testschema.tablespace_acl (c int);

CREATE INDEX k ON testschema.tablespace_acl (c) TABLESPACE regress_tblspace;

ALTER TABLE testschema.tablespace_acl OWNER TO regress_tablespace_user2;

SET SESSION ROLE regress_tablespace_user2;

CREATE TABLE tablespace_table (i int) TABLESPACE regress_tblspace;

ALTER TABLE testschema.tablespace_acl ALTER c TYPE bigint;

REINDEX (TABLESPACE regress_tblspace) TABLE tablespace_table;

REINDEX (TABLESPACE regress_tblspace, CONCURRENTLY) TABLE tablespace_table;

RESET ROLE;

ALTER TABLESPACE regress_tblspace RENAME TO regress_tblspace_renamed;

ALTER TABLE ALL IN TABLESPACE regress_tblspace_renamed SET TABLESPACE pg_default;

ALTER INDEX ALL IN TABLESPACE regress_tblspace_renamed SET TABLESPACE pg_default;

ALTER MATERIALIZED VIEW ALL IN TABLESPACE regress_tblspace_renamed SET TABLESPACE pg_default;

ALTER TABLE ALL IN TABLESPACE regress_tblspace_renamed SET TABLESPACE pg_default;

ALTER MATERIALIZED VIEW ALL IN TABLESPACE regress_tblspace_renamed SET TABLESPACE pg_default;

DROP TABLESPACE regress_tblspace_renamed;

DROP SCHEMA testschema CASCADE;

DROP ROLE regress_tablespace_user1;

DROP ROLE regress_tablespace_user2;
