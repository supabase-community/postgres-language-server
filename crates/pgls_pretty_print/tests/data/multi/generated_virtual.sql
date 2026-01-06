CREATE SCHEMA generated_virtual_tests;

GRANT USAGE ON SCHEMA generated_virtual_tests TO PUBLIC;

SET search_path = generated_virtual_tests;

SELECT table_name, column_name, column_default, is_nullable, is_generated, generation_expression FROM information_schema.columns WHERE table_schema = 'generated_virtual_tests' ORDER BY 1, 2;

SELECT table_name, column_name, dependent_column FROM information_schema.column_column_usage WHERE table_schema = 'generated_virtual_tests' ORDER BY 1, 2, 3;

DROP TABLE gtest2;

INSERT INTO gtest1 VALUES (1);

INSERT INTO gtest1 VALUES (2, DEFAULT);

INSERT INTO gtest1 VALUES (3, 33);

INSERT INTO gtest1 VALUES (3, 33), (4, 44);

INSERT INTO gtest1 VALUES (3, DEFAULT), (4, 44);

INSERT INTO gtest1 VALUES (3, 33), (4, DEFAULT);

INSERT INTO gtest1 VALUES (3, DEFAULT), (4, DEFAULT);

SELECT * FROM gtest1 ORDER BY a;

SELECT gtest1 FROM gtest1 ORDER BY a;

SELECT a, (SELECT gtest1.b) FROM gtest1 ORDER BY a;

DELETE FROM gtest1 WHERE a >= 3;

UPDATE gtest1 SET b = DEFAULT WHERE a = 1;

UPDATE gtest1 SET b = 11 WHERE a = 1;

SELECT * FROM gtest1 ORDER BY a;

SELECT a, b, b * 2 AS b2 FROM gtest1 ORDER BY a;

SELECT a, b FROM gtest1 WHERE b = 4 ORDER BY a;

INSERT INTO gtest1 VALUES (2000000000);

SELECT * FROM gtest1;

DELETE FROM gtest1 WHERE a = 2000000000;

CREATE TABLE gtestx (x int, y int);

INSERT INTO gtestx VALUES (11, 1), (22, 2), (33, 3);

SELECT * FROM gtestx, gtest1 WHERE gtestx.y = gtest1.a;

DROP TABLE gtestx;

SELECT * FROM gtest1 ORDER BY a;

UPDATE gtest1 SET a = 3 WHERE b = 4 RETURNING old.*, new.*;

SELECT * FROM gtest1 ORDER BY a;

DELETE FROM gtest1 WHERE b = 2;

SELECT * FROM gtest1 ORDER BY a;

INSERT INTO gtestm VALUES (1, 5, 100);

SELECT * FROM gtestm ORDER BY id;

DROP TABLE gtestm;

INSERT INTO gtestm (a) SELECT g FROM generate_series(1, 10) g;

DROP TABLE gtestm;

CREATE VIEW gtest1v AS SELECT * FROM gtest1;

SELECT * FROM gtest1v;

INSERT INTO gtest1v VALUES (4, 8);

INSERT INTO gtest1v VALUES (5, DEFAULT);

INSERT INTO gtest1v VALUES (6, 66), (7, 77);

INSERT INTO gtest1v VALUES (6, DEFAULT), (7, 77);

INSERT INTO gtest1v VALUES (6, 66), (7, DEFAULT);

INSERT INTO gtest1v VALUES (6, DEFAULT), (7, DEFAULT);

ALTER VIEW gtest1v ALTER COLUMN b SET DEFAULT 100;

INSERT INTO gtest1v VALUES (8, DEFAULT);

INSERT INTO gtest1v VALUES (8, DEFAULT), (9, DEFAULT);

SELECT * FROM gtest1v;

DELETE FROM gtest1v WHERE a >= 5;

DROP VIEW gtest1v;

WITH foo AS (SELECT * FROM gtest1) SELECT * FROM foo;

CREATE TABLE gtest1_1 () INHERITS (gtest1);

SELECT * FROM gtest1_1;

INSERT INTO gtest1_1 VALUES (4);

SELECT * FROM gtest1_1;

SELECT * FROM gtest1;

CREATE TABLE gtest_normal (a int, b int);

ALTER TABLE gtest_normal_child INHERIT gtest_normal;

DROP TABLE gtest_normal, gtest_normal_child;

CREATE TABLE gtestx (x int, b int DEFAULT 10) INHERITS (gtest1);

CREATE TABLE gtestx (x int, b int GENERATED ALWAYS AS IDENTITY) INHERITS (gtest1);

CREATE TABLE gtestx (x int, b int GENERATED ALWAYS AS (a * 22) STORED) INHERITS (gtest1);

INSERT INTO gtestx (a, x) VALUES (11, 22);

SELECT * FROM gtest1;

SELECT * FROM gtestx;

CREATE TABLE gtestxx_1 (a int NOT NULL, b int);

ALTER TABLE gtestxx_1 INHERIT gtest1;

ALTER TABLE gtestxx_3 INHERIT gtest1;

ALTER TABLE gtestxx_4 INHERIT gtest1;

CREATE TABLE gtesty (x int, b int DEFAULT 55);

CREATE TABLE gtest1_y () INHERITS (gtest0, gtesty);

DROP TABLE gtesty;

CREATE TABLE gtesty (x int, b int);

CREATE TABLE gtest1_y () INHERITS (gtest1, gtesty);

DROP TABLE gtesty;

CREATE TABLE gtest1_y () INHERITS (gtest1, gtesty);

CREATE TABLE gtestp (f1 int);

INSERT INTO gtestc values(42);

TABLE gtestc;

UPDATE gtestp SET f1 = f1 * 10;

TABLE gtestc;

DROP TABLE gtestp CASCADE;

INSERT INTO gtest3 (a) VALUES (1), (2), (3), (NULL);

SELECT * FROM gtest3 ORDER BY a;

UPDATE gtest3 SET a = 22 WHERE a = 2;

SELECT * FROM gtest3 ORDER BY a;

INSERT INTO gtest3a (a) VALUES ('a'), ('b'), ('c'), (NULL);

SELECT * FROM gtest3a ORDER BY a;

UPDATE gtest3a SET a = 'bb' WHERE a = 'b';

SELECT * FROM gtest3a ORDER BY a;

TRUNCATE gtest1;

INSERT INTO gtest1 (a) VALUES (1), (2);

COPY gtest1 TO stdout;

COPY gtest1 (a, b) TO stdout;

SELECT * FROM gtest1 ORDER BY a;

TRUNCATE gtest3;

INSERT INTO gtest3 (a) VALUES (1), (2);

COPY gtest3 TO stdout;

COPY gtest3 (a, b) TO stdout;

SELECT * FROM gtest3 ORDER BY a;

INSERT INTO gtest2 VALUES (1);

SELECT * FROM gtest2;

INSERT INTO gtest_varlena (a) VALUES('01234567890123456789');

INSERT INTO gtest_varlena (a) VALUES(NULL);

SELECT * FROM gtest_varlena ORDER BY a;

DROP TABLE gtest_varlena;

CREATE TYPE double_int as (a int, b int);

DROP TYPE double_int;

INSERT INTO gtest_tableoid VALUES (1), (2);

SELECT * FROM gtest_tableoid;

ALTER TABLE gtest10 DROP COLUMN b;

ALTER TABLE gtest10 DROP COLUMN b CASCADE;

ALTER TABLE gtest10a DROP COLUMN b;

INSERT INTO gtest10a (a) VALUES (1);

CREATE USER regress_user11;

INSERT INTO gtest11 VALUES (1, 10), (2, 20);

GRANT SELECT (a, c) ON gtest11 TO regress_user11;

CREATE FUNCTION gf1(a int) RETURNS int AS $$ SELECT a * 3 $$ IMMUTABLE LANGUAGE SQL;

REVOKE ALL ON FUNCTION gf1(int) FROM PUBLIC;

SET ROLE regress_user11;

SELECT a, b FROM gtest11;

SELECT a, c FROM gtest11;

SELECT gf1(10);

RESET ROLE;

DROP TABLE gtest11;

DROP FUNCTION gf1(int);

DROP USER regress_user11;

INSERT INTO gtest20 (a) VALUES (10);

INSERT INTO gtest20 (a) VALUES (30);

ALTER TABLE gtest20 ALTER COLUMN b SET EXPRESSION AS (a * 100);

ALTER TABLE gtest20 ALTER COLUMN b SET EXPRESSION AS (a * 3);

INSERT INTO gtest20a (a) VALUES (10);

INSERT INTO gtest20a (a) VALUES (30);

ALTER TABLE gtest20a ADD CHECK (b < 50);

ALTER TABLE gtest20a ADD COLUMN c float8 DEFAULT random() CHECK (b < 50);

ALTER TABLE gtest20a ADD COLUMN c float8 DEFAULT random() CHECK (b < 61);

INSERT INTO gtest20b (a) VALUES (10);

INSERT INTO gtest20b (a) VALUES (30);

ALTER TABLE gtest20b ADD CONSTRAINT chk CHECK (b < 50) NOT VALID;

ALTER TABLE gtest20b VALIDATE CONSTRAINT chk;

ALTER TABLE gtest20c ADD CONSTRAINT whole_row_check CHECK (gtest20c IS NOT NULL);

INSERT INTO gtest20c VALUES (1);

INSERT INTO gtest20c VALUES (NULL);

INSERT INTO gtest21a (a) VALUES (1);

INSERT INTO gtest21a (a) VALUES (0);

INSERT INTO gtest21ax (a) VALUES (0);

INSERT INTO gtest21ax (a) VALUES (1);

ALTER TABLE gtest21ax ALTER COLUMN b SET EXPRESSION AS (nullif(a, 1));

DROP TABLE gtest21ax;

INSERT INTO gtest21ax (a) VALUES (0);

DROP TABLE gtest21ax;

ALTER TABLE gtest21b ALTER COLUMN b SET NOT NULL;

INSERT INTO gtest21b (a) VALUES (1);

INSERT INTO gtest21b (a) VALUES (2), (0);

INSERT INTO gtest21b (a) VALUES (NULL);

ALTER TABLE gtest21b ALTER COLUMN b DROP NOT NULL;

INSERT INTO gtest21b (a) VALUES (0);

CREATE TABLE gtestnn_child PARTITION OF gtestnn_parent FOR VALUES FROM (1) TO (5);

CREATE TABLE gtestnn_childdef PARTITION OF gtestnn_parent default;

INSERT INTO gtestnn_parent VALUES (2, 2, default), (3, 5, default), (14, 12, default);

INSERT INTO gtestnn_parent VALUES (1, 2, default);

INSERT INTO gtestnn_parent VALUES (2, 10, default);

ALTER TABLE gtestnn_parent ALTER COLUMN f3 SET EXPRESSION AS (nullif(f1, 2) + nullif(f2, 11));

INSERT INTO gtestnn_parent VALUES (10, 11, default);

SELECT * FROM gtestnn_parent ORDER BY f1, f2, f3;

CREATE TABLE gtest23a (x int PRIMARY KEY, y int);

CREATE TABLE gtest23q (a int PRIMARY KEY, b int REFERENCES gtest23p (y));

CREATE DOMAIN gtestdomain1 AS int CHECK (VALUE < 10);

CREATE TYPE gtestdomain1range AS range (subtype = gtestdomain1);

CREATE TABLE gtest24at (a int PRIMARY KEY);

ALTER TABLE gtest24ata ALTER COLUMN b TYPE gtestdomain1;

CREATE DOMAIN gtestdomainnn AS int CHECK (VALUE IS NOT NULL);

CREATE TYPE gtest_type AS (f1 integer, f2 text, f3 bigint);

DROP TYPE gtest_type CASCADE;

CREATE TABLE gtest_parent (f1 date NOT NULL, f2 bigint, f3 bigint) PARTITION BY RANGE (f1);

ALTER TABLE gtest_parent ATTACH PARTITION gtest_child FOR VALUES FROM ('2016-07-01') TO ('2016-08-01');

DROP TABLE gtest_parent, gtest_child;

CREATE TABLE gtest_child PARTITION OF gtest_parent
  FOR VALUES FROM ('2016-07-01') TO ('2016-08-01');

CREATE TABLE gtest_child3 PARTITION OF gtest_parent (
    f3 DEFAULT 42  -- error
) FOR VALUES FROM ('2016-09-01') TO ('2016-10-01');

CREATE TABLE gtest_child3 PARTITION OF gtest_parent (
    f3 WITH OPTIONS GENERATED ALWAYS AS IDENTITY  -- error
) FOR VALUES FROM ('2016-09-01') TO ('2016-10-01');

CREATE TABLE gtest_child3 PARTITION OF gtest_parent (
    f3 GENERATED ALWAYS AS (f2 * 2) STORED  -- error
) FOR VALUES FROM ('2016-09-01') TO ('2016-10-01');

CREATE TABLE gtest_child3 (f1 date NOT NULL, f2 bigint, f3 bigint);

ALTER TABLE gtest_parent ATTACH PARTITION gtest_child3 FOR VALUES FROM ('2016-09-01') TO ('2016-10-01');

DROP TABLE gtest_child3;

CREATE TABLE gtest_child3 (f1 date NOT NULL, f2 bigint, f3 bigint DEFAULT 42);

ALTER TABLE gtest_parent ATTACH PARTITION gtest_child3 FOR VALUES FROM ('2016-09-01') TO ('2016-10-01');

DROP TABLE gtest_child3;

CREATE TABLE gtest_child3 (f1 date NOT NULL, f2 bigint, f3 bigint GENERATED ALWAYS AS IDENTITY);

ALTER TABLE gtest_parent ATTACH PARTITION gtest_child3 FOR VALUES FROM ('2016-09-01') TO ('2016-10-01');

DROP TABLE gtest_child3;

CREATE TABLE gtest_child3 (f1 date NOT NULL, f2 bigint, f3 bigint GENERATED ALWAYS AS (f2 * 33) STORED);

ALTER TABLE gtest_parent ATTACH PARTITION gtest_child3 FOR VALUES FROM ('2016-09-01') TO ('2016-10-01');

DROP TABLE gtest_child3;

ALTER TABLE gtest_parent ATTACH PARTITION gtest_child3 FOR VALUES FROM ('2016-09-01') TO ('2016-10-01');

INSERT INTO gtest_parent (f1, f2) VALUES ('2016-07-15', 1);

INSERT INTO gtest_parent (f1, f2) VALUES ('2016-07-15', 2);

INSERT INTO gtest_parent (f1, f2) VALUES ('2016-08-15', 3);

SELECT tableoid::regclass, * FROM gtest_parent ORDER BY 1, 2, 3;

SELECT tableoid::regclass, * FROM gtest_child ORDER BY 1, 2, 3;

SELECT tableoid::regclass, * FROM gtest_child2 ORDER BY 1, 2, 3;

SELECT tableoid::regclass, * FROM gtest_child3 ORDER BY 1, 2, 3;

UPDATE gtest_parent SET f1 = f1 + 60 WHERE f2 = 1;

SELECT tableoid::regclass, * FROM gtest_parent ORDER BY 1, 2, 3;

ALTER TABLE ONLY gtest_parent ALTER COLUMN f3 SET EXPRESSION AS (f2 * 4);

ALTER TABLE gtest_child ALTER COLUMN f3 SET EXPRESSION AS (f2 * 10);

SELECT tableoid::regclass, * FROM gtest_parent ORDER BY 1, 2, 3;

ALTER TABLE gtest_parent ALTER COLUMN f3 SET EXPRESSION AS (f2 * 2);

SELECT tableoid::regclass, * FROM gtest_parent ORDER BY 1, 2, 3;

CREATE TABLE gtest25 (a int PRIMARY KEY);

INSERT INTO gtest25 VALUES (3), (4);

SELECT * FROM gtest25 ORDER BY a;

ALTER TABLE gtest25 ADD COLUMN d int DEFAULT 101;

SELECT * FROM gtest25 ORDER BY a;

INSERT INTO gtest27 (a, b) VALUES (3, 7), (4, 11);

ALTER TABLE gtest27 ALTER COLUMN a TYPE text;

ALTER TABLE gtest27 ALTER COLUMN x TYPE numeric;

SELECT * FROM gtest27;

ALTER TABLE gtest27 ALTER COLUMN x TYPE boolean USING x <> 0;

ALTER TABLE gtest27 ALTER COLUMN x DROP DEFAULT;

INSERT INTO gtest27 (a, b) VALUES (NULL, NULL);

DELETE FROM gtest27 WHERE a IS NULL AND b IS NULL;

ALTER TABLE gtest27
  ALTER COLUMN a TYPE float8,
  ALTER COLUMN b TYPE float8;

SELECT * FROM gtest27;

INSERT INTO gtest29 (a) VALUES (3), (4);

SELECT * FROM gtest29;

ALTER TABLE gtest29 ALTER COLUMN a SET EXPRESSION AS (a * 3);

ALTER TABLE gtest29 ALTER COLUMN a DROP EXPRESSION;

ALTER TABLE gtest29 ALTER COLUMN a DROP EXPRESSION IF EXISTS;

ALTER TABLE gtest29 ALTER COLUMN b SET EXPRESSION AS (a * 3);

SELECT * FROM gtest29;

ALTER TABLE gtest29 ALTER COLUMN b DROP EXPRESSION;

INSERT INTO gtest29 (a) VALUES (5);

INSERT INTO gtest29 (a, b) VALUES (6, 66);

SELECT * FROM gtest29;

CREATE TABLE gtest30_1 () INHERITS (gtest30);

ALTER TABLE gtest30 ALTER COLUMN b DROP EXPRESSION;

DROP TABLE gtest30 CASCADE;

CREATE TABLE gtest30_1 () INHERITS (gtest30);

ALTER TABLE ONLY gtest30 ALTER COLUMN b DROP EXPRESSION;

ALTER TABLE gtest30_1 ALTER COLUMN b DROP EXPRESSION;

CREATE TABLE gtest31_2 (x int, y gtest31_1);

ALTER TABLE gtest31_1 ALTER COLUMN b TYPE varchar;

ALTER TABLE gtest31_2 ADD CONSTRAINT cc CHECK ((y).b IS NOT NULL);

ALTER TABLE gtest31_1 ALTER COLUMN b SET EXPRESSION AS ('hello1');

ALTER TABLE gtest31_2 DROP CONSTRAINT cc;

CREATE STATISTICS gtest31_2_stat ON ((y).b is not null) FROM gtest31_2;

ALTER TABLE gtest31_1 ALTER COLUMN b SET EXPRESSION AS ('hello2');

DROP STATISTICS gtest31_2_stat;

CREATE INDEX gtest31_2_y_idx ON gtest31_2(((y).b));

ALTER TABLE gtest31_1 ALTER COLUMN b SET EXPRESSION AS ('hello3');

DROP TABLE gtest31_1, gtest31_2;

CREATE TABLE gtest31_2 (x int, y gtest31_1);

ALTER TABLE gtest31_1 ALTER COLUMN b TYPE varchar;

DROP TABLE gtest31_1, gtest31_2;

CREATE FUNCTION gtest_trigger_func() RETURNS trigger
  LANGUAGE plpgsql
AS $$
BEGIN
  IF tg_op IN ('DELETE', 'UPDATE') THEN
    RAISE INFO '%: %: old = %', TG_NAME, TG_WHEN, OLD;
  END IF;
  IF tg_op IN ('INSERT', 'UPDATE') THEN
    RAISE INFO '%: %: new = %', TG_NAME, TG_WHEN, NEW;
  END IF;
  IF tg_op = 'DELETE' THEN
    RETURN OLD;
  ELSE
    RETURN NEW;
  END IF;
END
$$;

CREATE TRIGGER gtest1 BEFORE DELETE OR UPDATE ON gtest26
  FOR EACH ROW
  WHEN (OLD.b < 0)  -- ok
  EXECUTE PROCEDURE gtest_trigger_func();

CREATE TRIGGER gtest2a BEFORE INSERT OR UPDATE ON gtest26
  FOR EACH ROW
  WHEN (NEW.b < 0)  -- error
  EXECUTE PROCEDURE gtest_trigger_func();

CREATE TRIGGER gtest2b BEFORE INSERT OR UPDATE ON gtest26
  FOR EACH ROW
  WHEN (NEW.* IS NOT NULL)  -- error
  EXECUTE PROCEDURE gtest_trigger_func();

CREATE TRIGGER gtest2 BEFORE INSERT ON gtest26
  FOR EACH ROW
  WHEN (NEW.a < 0)
  EXECUTE PROCEDURE gtest_trigger_func();

CREATE TRIGGER gtest3 AFTER DELETE OR UPDATE ON gtest26
  FOR EACH ROW
  WHEN (OLD.b < 0)  -- ok
  EXECUTE PROCEDURE gtest_trigger_func();

CREATE TRIGGER gtest4 AFTER INSERT OR UPDATE ON gtest26
  FOR EACH ROW
  WHEN (NEW.b < 0)  -- ok
  EXECUTE PROCEDURE gtest_trigger_func();

INSERT INTO gtest26 (a) VALUES (-2), (0), (3);

SELECT * FROM gtest26 ORDER BY a;

UPDATE gtest26 SET a = a * -2;

SELECT * FROM gtest26 ORDER BY a;

DELETE FROM gtest26 WHERE a = -6;

SELECT * FROM gtest26 ORDER BY a;

DROP TRIGGER gtest1 ON gtest26;

DROP TRIGGER gtest2 ON gtest26;

DROP TRIGGER gtest3 ON gtest26;

CREATE FUNCTION gtest_trigger_func3() RETURNS trigger
  LANGUAGE plpgsql
AS $$
BEGIN
  RAISE NOTICE 'OK';
  RETURN NEW;
END
$$;

CREATE TRIGGER gtest11 BEFORE UPDATE OF b ON gtest26
  FOR EACH ROW
  EXECUTE PROCEDURE gtest_trigger_func3();

UPDATE gtest26 SET a = 1 WHERE a = 0;

DROP TRIGGER gtest11 ON gtest26;

TRUNCATE gtest26;

CREATE FUNCTION gtest_trigger_func4() RETURNS trigger
  LANGUAGE plpgsql
AS $$
BEGIN
  NEW.a = 10;
  NEW.b = 300;
  RETURN NEW;
END;
$$;

CREATE TRIGGER gtest12_01 BEFORE INSERT OR UPDATE ON gtest26
  FOR EACH ROW
  EXECUTE PROCEDURE gtest_trigger_func();

CREATE TRIGGER gtest12_02 BEFORE INSERT OR UPDATE ON gtest26
  FOR EACH ROW
  EXECUTE PROCEDURE gtest_trigger_func4();

CREATE TRIGGER gtest12_03 BEFORE INSERT OR UPDATE ON gtest26
  FOR EACH ROW
  EXECUTE PROCEDURE gtest_trigger_func();

INSERT INTO gtest26 (a) VALUES (1);

SELECT * FROM gtest26 ORDER BY a;

UPDATE gtest26 SET a = 11 WHERE a = 10;

SELECT * FROM gtest26 ORDER BY a;

ALTER TABLE gtest28a DROP COLUMN a;

CREATE TABLE gtest28b (LIKE gtest28a INCLUDING GENERATED);

SELECT attrelid, attname, attgenerated FROM pg_attribute WHERE attgenerated NOT IN ('', 's', 'v');

insert into gtest32 values (1), (2);

analyze gtest32;

select sum(t2.b) over (partition by t2.a),
       sum(t2.c) over (partition by t2.a),
       sum(t2.d) over (partition by t2.a)
from gtest32 as t1 left join gtest32 as t2 on (t1.a = t2.a)
order by t1.a;

select sum(t2.b) over (partition by t2.a),
       sum(t2.c) over (partition by t2.a),
       sum(t2.d) over (partition by t2.a)
from gtest32 as t1 left join gtest32 as t2 on (t1.a = t2.a)
order by t1.a;

select t1.a from gtest32 t1 left join gtest32 t2 on t1.a = t2.a
where coalesce(t2.b, 1) = 2;

select t1.a from gtest32 t1 left join gtest32 t2 on t1.a = t2.a
where coalesce(t2.b, 1) = 2;

select t1.a from gtest32 t1 left join gtest32 t2 on t1.a = t2.a
where coalesce(t2.b, 1) = 2 or t1.a is null;

select t1.a from gtest32 t1 left join gtest32 t2 on t1.a = t2.a
where coalesce(t2.b, 1) = 2 or t1.a is null;

select t2.* from gtest32 t1 left join gtest32 t2 on false;

select t2.* from gtest32 t1 left join gtest32 t2 on false;

select * from gtest32 t group by grouping sets (a, b, c, d, e) having c = 20;

select * from gtest32 t group by grouping sets (a, b, c, d, e) having c = 20;

alter table gtest32 alter column e type bigint using b;

select 1 from gtest32 t1 where exists
  (select 1 from gtest32 t2 where t1.a > t2.a and t2.b = 2);

select 1 from gtest32 t1 where exists
  (select 1 from gtest32 t2 where t1.a > t2.a and t2.b = 2);

drop table gtest32;

set constraint_exclusion to on;

select * from gtest33 where b < 10;

select * from gtest33 where b is null;

reset constraint_exclusion;

drop table gtest33;
