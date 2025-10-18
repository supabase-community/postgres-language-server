SELECT *
   INTO TABLE sitmp1
   FROM onek
   WHERE onek.unique1 < 2;

DROP TABLE sitmp1;

SELECT *
   INTO TABLE sitmp1
   FROM onek2
   WHERE onek2.unique1 < 2;

DROP TABLE sitmp1;

CREATE SCHEMA selinto_schema;

CREATE USER regress_selinto_user;

ALTER DEFAULT PRIVILEGES FOR ROLE regress_selinto_user
	  REVOKE INSERT ON TABLES FROM regress_selinto_user;

GRANT ALL ON SCHEMA selinto_schema TO public;

SET SESSION AUTHORIZATION regress_selinto_user;

CREATE TABLE selinto_schema.tbl_withdata1 (a)
  AS SELECT generate_series(1,3) WITH DATA;

INSERT INTO selinto_schema.tbl_withdata1 VALUES (4);

CREATE TABLE selinto_schema.tbl_withdata2 (a) AS
  SELECT generate_series(1,3) WITH DATA;

CREATE TABLE selinto_schema.tbl_nodata1 (a) AS
  SELECT generate_series(1,3) WITH NO DATA;

CREATE TABLE selinto_schema.tbl_nodata2 (a) AS
  SELECT generate_series(1,3) WITH NO DATA;

PREPARE data_sel AS SELECT generate_series(1,3);

CREATE TABLE selinto_schema.tbl_withdata3 (a) AS
  EXECUTE data_sel WITH DATA;

CREATE TABLE selinto_schema.tbl_withdata4 (a) AS
  EXECUTE data_sel WITH DATA;

CREATE TABLE selinto_schema.tbl_nodata3 (a) AS
  EXECUTE data_sel WITH NO DATA;

CREATE TABLE selinto_schema.tbl_nodata4 (a) AS
  EXECUTE data_sel WITH NO DATA;

RESET SESSION AUTHORIZATION;

ALTER DEFAULT PRIVILEGES FOR ROLE regress_selinto_user
	  GRANT INSERT ON TABLES TO regress_selinto_user;

SET SESSION AUTHORIZATION regress_selinto_user;

RESET SESSION AUTHORIZATION;

DEALLOCATE data_sel;

DROP SCHEMA selinto_schema CASCADE;

DROP USER regress_selinto_user;

CREATE TABLE ctas_base (i int, j int);

INSERT INTO ctas_base VALUES (1, 2);

CREATE TABLE ctas_nodata (ii, jj, kk) AS SELECT i, j FROM ctas_base;

CREATE TABLE ctas_nodata (ii, jj, kk) AS SELECT i, j FROM ctas_base WITH NO DATA;

CREATE TABLE ctas_nodata (ii, jj) AS SELECT i, j FROM ctas_base;

CREATE TABLE ctas_nodata_2 (ii, jj) AS SELECT i, j FROM ctas_base WITH NO DATA;

CREATE TABLE ctas_nodata_3 (ii) AS SELECT i, j FROM ctas_base;

CREATE TABLE ctas_nodata_4 (ii) AS SELECT i, j FROM ctas_base WITH NO DATA;

SELECT * FROM ctas_nodata;

SELECT * FROM ctas_nodata_2;

SELECT * FROM ctas_nodata_3;

SELECT * FROM ctas_nodata_4;

DROP TABLE ctas_base;

DROP TABLE ctas_nodata;

DROP TABLE ctas_nodata_2;

DROP TABLE ctas_nodata_3;

DROP TABLE ctas_nodata_4;

CREATE FUNCTION make_table() RETURNS VOID
AS $$
  CREATE TABLE created_table AS SELECT * FROM int8_tbl;
$$ LANGUAGE SQL;

SELECT make_table();

SELECT * FROM created_table;

DO $$
BEGIN
	EXECUTE 'EXPLAIN ANALYZE SELECT * INTO TABLE easi FROM int8_tbl';
	EXECUTE 'EXPLAIN ANALYZE CREATE TABLE easi2 AS SELECT * FROM int8_tbl WITH NO DATA';
END$$;

DROP TABLE created_table;

DROP TABLE easi, easi2;

DECLARE foo CURSOR FOR SELECT 1 INTO int4_tbl;

COPY (SELECT 1 INTO frak UNION SELECT 2) TO 'blob';

SELECT * FROM (SELECT 1 INTO f) bar;

CREATE VIEW foo AS SELECT 1 INTO int4_tbl;

INSERT INTO int4_tbl SELECT 1 INTO f;

CREATE TABLE ctas_ine_tbl AS SELECT 1;

CREATE TABLE ctas_ine_tbl AS SELECT 1 / 0;

CREATE TABLE IF NOT EXISTS ctas_ine_tbl AS SELECT 1 / 0;

CREATE TABLE ctas_ine_tbl AS SELECT 1 / 0 WITH NO DATA;

CREATE TABLE IF NOT EXISTS ctas_ine_tbl AS SELECT 1 / 0 WITH NO DATA;

CREATE TABLE ctas_ine_tbl AS SELECT 1 / 0;

CREATE TABLE IF NOT EXISTS ctas_ine_tbl AS SELECT 1 / 0;

CREATE TABLE ctas_ine_tbl AS SELECT 1 / 0 WITH NO DATA;

CREATE TABLE IF NOT EXISTS ctas_ine_tbl AS SELECT 1 / 0 WITH NO DATA;

PREPARE ctas_ine_query AS SELECT 1 / 0;

CREATE TABLE ctas_ine_tbl AS EXECUTE ctas_ine_query;

CREATE TABLE IF NOT EXISTS ctas_ine_tbl AS EXECUTE ctas_ine_query;

DROP TABLE ctas_ine_tbl;
