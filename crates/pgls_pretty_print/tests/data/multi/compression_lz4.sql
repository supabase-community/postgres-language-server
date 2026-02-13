SELECT NOT(enumvals @> '{lz4}') AS skip_test FROM pg_settings WHERE
  name = 'default_toast_compression' ;

CREATE SCHEMA lz4;

SET search_path TO lz4, public;

SET default_toast_compression = 'pglz';

CREATE TABLE cmdata_pglz(f1 text COMPRESSION pglz);

CREATE INDEX idx ON cmdata_pglz(f1);

INSERT INTO cmdata_pglz VALUES(repeat('1234567890', 1000));

CREATE TABLE cmdata_lz4(f1 TEXT COMPRESSION lz4);

INSERT INTO cmdata_lz4 VALUES(repeat('1234567890', 1004));

SELECT pg_column_compression(f1) FROM cmdata_lz4;

SELECT SUBSTR(f1, 200, 5) FROM cmdata_pglz;

SELECT SUBSTR(f1, 2000, 50) FROM cmdata_lz4;

SELECT * INTO cmmove1 FROM cmdata_lz4;

SELECT pg_column_compression(f1) FROM cmmove1;

CREATE TABLE cmdata2 (LIKE cmdata_lz4 INCLUDING COMPRESSION);

DROP TABLE cmdata2;

CREATE TABLE cmmove3(f1 text COMPRESSION pglz);

INSERT INTO cmmove3 SELECT * FROM cmdata_pglz;

INSERT INTO cmmove3 SELECT * FROM cmdata_lz4;

SELECT pg_column_compression(f1) FROM cmmove3;

CREATE TABLE cmmove2(f1 text COMPRESSION pglz);

INSERT INTO cmmove2 VALUES (repeat('1234567890', 1004));

SELECT pg_column_compression(f1) FROM cmmove2;

UPDATE cmmove2 SET f1 = cmdata_lz4.f1 FROM cmdata_lz4;

SELECT pg_column_compression(f1) FROM cmmove2;

CREATE OR REPLACE FUNCTION large_val_lz4() RETURNS TEXT LANGUAGE SQL AS
'select array_agg(fipshash(g::text))::text from generate_series(1, 256) g';

CREATE TABLE cmdata2 (f1 text COMPRESSION lz4);

INSERT INTO cmdata2 SELECT large_val_lz4() || repeat('a', 4000);

SELECT pg_column_compression(f1) FROM cmdata2;

SELECT SUBSTR(f1, 200, 5) FROM cmdata2;

DROP TABLE cmdata2;

DROP FUNCTION large_val_lz4;

CREATE MATERIALIZED VIEW compressmv(x) AS SELECT * FROM cmdata_lz4;

SELECT pg_column_compression(f1) FROM cmdata_lz4;

SELECT pg_column_compression(x) FROM compressmv;

CREATE TABLE cmpart(f1 text COMPRESSION lz4) PARTITION BY HASH(f1);

CREATE TABLE cmpart1 PARTITION OF cmpart FOR VALUES WITH (MODULUS 2, REMAINDER 0);

CREATE TABLE cmpart2(f1 text COMPRESSION pglz);

INSERT INTO cmpart VALUES (repeat('123456789', 1004));

INSERT INTO cmpart VALUES (repeat('123456789', 4004));

SELECT pg_column_compression(f1) FROM cmpart1;

SELECT pg_column_compression(f1) FROM cmpart2;

CREATE TABLE cminh() INHERITS(cmdata_pglz, cmdata_lz4);

CREATE TABLE cminh(f1 TEXT COMPRESSION lz4) INHERITS(cmdata_pglz);

CREATE TABLE cmdata3(f1 text);

CREATE TABLE cminh() INHERITS (cmdata_pglz, cmdata3);

SET default_toast_compression = 'lz4';

ALTER TABLE cmdata_pglz ALTER COLUMN f1 SET COMPRESSION lz4;

INSERT INTO cmdata_pglz VALUES (repeat('123456789', 4004));

SELECT pg_column_compression(f1) FROM cmdata_pglz;

ALTER TABLE cmdata_pglz ALTER COLUMN f1 SET COMPRESSION pglz;

ALTER MATERIALIZED VIEW compressmv ALTER COLUMN x SET COMPRESSION lz4;

ALTER TABLE cmpart1 ALTER COLUMN f1 SET COMPRESSION pglz;

ALTER TABLE cmpart2 ALTER COLUMN f1 SET COMPRESSION lz4;

INSERT INTO cmpart VALUES (repeat('123456789', 1004));

INSERT INTO cmpart VALUES (repeat('123456789', 4004));

SELECT pg_column_compression(f1) FROM cmpart1;

SELECT pg_column_compression(f1) FROM cmpart2;

CREATE TABLE cmdata2 (f1 TEXT COMPRESSION pglz, f2 TEXT COMPRESSION lz4);

CREATE UNIQUE INDEX idx1 ON cmdata2 ((f1 || f2));

INSERT INTO cmdata2 VALUES((SELECT array_agg(fipshash(g::TEXT))::TEXT FROM
generate_series(1, 50) g), VERSION());

SELECT length(f1) FROM cmdata_pglz;

SELECT length(f1) FROM cmdata_lz4;

SELECT length(f1) FROM cmmove1;

SELECT length(f1) FROM cmmove2;

SELECT length(f1) FROM cmmove3;
