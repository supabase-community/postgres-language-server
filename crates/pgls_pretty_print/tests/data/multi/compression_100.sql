CREATE SCHEMA pglz;

SET search_path TO pglz, public;

SET default_toast_compression = 'pglz';

CREATE TABLE cmdata(f1 text COMPRESSION pglz);

CREATE INDEX idx ON cmdata(f1);

INSERT INTO cmdata VALUES(repeat('1234567890', 1000));

SELECT pg_column_compression(f1) FROM cmdata;

SELECT SUBSTR(f1, 200, 5) FROM cmdata;

SELECT * INTO cmmove1 FROM cmdata;

SELECT pg_column_compression(f1) FROM cmmove1;

CREATE TABLE cmdata2 (f1 int COMPRESSION pglz);

CREATE OR REPLACE FUNCTION large_val() RETURNS TEXT LANGUAGE SQL AS
'select array_agg(fipshash(g::text))::text from generate_series(1, 256) g';

CREATE TABLE cmdata2 (f1 text COMPRESSION pglz);

INSERT INTO cmdata2 SELECT large_val() || repeat('a', 4000);

SELECT pg_column_compression(f1) FROM cmdata2;

SELECT SUBSTR(f1, 200, 5) FROM cmdata2;

DROP TABLE cmdata2;

CREATE TABLE cmdata2 (f1 int);

ALTER TABLE cmdata2 ALTER COLUMN f1 TYPE varchar;

ALTER TABLE cmdata2 ALTER COLUMN f1 TYPE int USING f1::integer;

ALTER TABLE cmdata2 ALTER COLUMN f1 TYPE varchar;

ALTER TABLE cmdata2 ALTER COLUMN f1 SET COMPRESSION pglz;

ALTER TABLE cmdata2 ALTER COLUMN f1 SET STORAGE plain;

INSERT INTO cmdata2 VALUES (repeat('123456789', 800));

SELECT pg_column_compression(f1) FROM cmdata2;

CREATE TABLE cmdata3(f1 text);

CREATE TABLE cminh() INHERITS (cmdata, cmdata3);

SET default_toast_compression = '';

SET default_toast_compression = 'I do not exist compression';

SET default_toast_compression = 'pglz';

ALTER TABLE cmdata2 ALTER COLUMN f1 SET COMPRESSION default;

DROP TABLE cmdata2;

SELECT pg_column_compression(f1) FROM cmdata;

VACUUM FULL cmdata;

SELECT pg_column_compression(f1) FROM cmdata;

SELECT length(f1) FROM cmdata;

SELECT length(f1) FROM cmmove1;

CREATE TABLE badcompresstbl (a text COMPRESSION I_Do_Not_Exist_Compression);

CREATE TABLE badcompresstbl (a text);

ALTER TABLE badcompresstbl ALTER a SET COMPRESSION I_Do_Not_Exist_Compression;

DROP TABLE badcompresstbl;
