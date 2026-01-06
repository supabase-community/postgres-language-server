CREATE TABLE tbl_gist (c1 int, c2 int, c3 int, c4 box);

INSERT INTO tbl_gist SELECT x, 2*x, 3*x, box(point(x,x+1),point(2*x,2*x+1)) FROM generate_series(1,8000) AS x;

CREATE INDEX tbl_gist_idx ON tbl_gist using gist (c4) INCLUDE (c1,c2,c3);

SELECT pg_get_indexdef(i.indexrelid)
FROM pg_index i JOIN pg_class c ON i.indexrelid = c.oid
WHERE i.indrelid = 'tbl_gist'::regclass ORDER BY c.relname;

SELECT * FROM tbl_gist where c4 <@ box(point(1,1),point(10,10));

SET enable_bitmapscan TO off;

SELECT * FROM tbl_gist where c4 <@ box(point(1,1),point(10,10));

SET enable_bitmapscan TO default;

DROP TABLE tbl_gist;

CREATE TABLE tbl_gist (c1 int, c2 int, c3 int, c4 box);

CREATE INDEX tbl_gist_idx ON tbl_gist using gist (c4) INCLUDE (c1,c2,c3);

INSERT INTO tbl_gist SELECT x, 2*x, 3*x, box(point(x,x+1),point(2*x,2*x+1)) FROM generate_series(1,8000) AS x;

SELECT pg_get_indexdef(i.indexrelid)
FROM pg_index i JOIN pg_class c ON i.indexrelid = c.oid
WHERE i.indrelid = 'tbl_gist'::regclass ORDER BY c.relname;

SELECT * FROM tbl_gist where c4 <@ box(point(1,1),point(10,10));

SET enable_bitmapscan TO off;

SELECT * FROM tbl_gist where c4 <@ box(point(1,1),point(10,10));

SET enable_bitmapscan TO default;

DROP TABLE tbl_gist;

CREATE TABLE tbl_gist (c1 int, c2 int, c3 int, c4 box);

INSERT INTO tbl_gist SELECT x, 2*x, 3*x, box(point(x,x+1),point(2*x,2*x+1)) FROM generate_series(1,10) AS x;

CREATE INDEX CONCURRENTLY tbl_gist_idx ON tbl_gist using gist (c4) INCLUDE (c1,c2,c3);

SELECT indexdef FROM pg_indexes WHERE tablename = 'tbl_gist' ORDER BY indexname;

DROP TABLE tbl_gist;

CREATE TABLE tbl_gist (c1 int, c2 int, c3 int, c4 box);

INSERT INTO tbl_gist SELECT x, 2*x, 3*x, box(point(x,x+1),point(2*x,2*x+1)) FROM generate_series(1,10) AS x;

CREATE INDEX tbl_gist_idx ON tbl_gist using gist (c4) INCLUDE (c1,c3);

SELECT indexdef FROM pg_indexes WHERE tablename = 'tbl_gist' ORDER BY indexname;

REINDEX INDEX tbl_gist_idx;

SELECT indexdef FROM pg_indexes WHERE tablename = 'tbl_gist' ORDER BY indexname;

ALTER TABLE tbl_gist DROP COLUMN c1;

SELECT indexdef FROM pg_indexes WHERE tablename = 'tbl_gist' ORDER BY indexname;

DROP TABLE tbl_gist;

CREATE TABLE tbl_gist (c1 int, c2 int, c3 int, c4 box);

INSERT INTO tbl_gist SELECT x, 2*x, 3*x, box(point(x,x+1),point(2*x,2*x+1)) FROM generate_series(1,10) AS x;

CREATE INDEX tbl_gist_idx ON tbl_gist using gist (c4) INCLUDE (c1,c3);

UPDATE tbl_gist SET c1 = 100 WHERE c1 = 2;

UPDATE tbl_gist SET c1 = 1 WHERE c1 = 3;

DELETE FROM tbl_gist WHERE c1 = 5 OR c3 = 12;

DROP TABLE tbl_gist;

CREATE TABLE tbl_gist (c1 int, c2 int, c3 int, c4 box);

INSERT INTO tbl_gist SELECT x, 2*x, 3*x, box(point(x,x+1),point(2*x,2*x+1)) FROM generate_series(1,10) AS x;

CREATE INDEX tbl_gist_idx ON tbl_gist using gist (c4) INCLUDE (c1,c3);

ALTER TABLE tbl_gist ALTER c1 TYPE bigint;

ALTER TABLE tbl_gist ALTER c3 TYPE bigint;

DROP TABLE tbl_gist;

CREATE TABLE tbl_gist (c1 int, c2 int, c3 int, c4 box, EXCLUDE USING gist (c4 WITH &&) INCLUDE (c1, c2, c3));

INSERT INTO tbl_gist SELECT x, 2*x, 3*x, box(point(x,x+1),point(2*x,2*x+1)) FROM generate_series(1,10) AS x;

INSERT INTO tbl_gist SELECT x, 2*x, 3*x, box(point(3*x,2*x),point(3*x+1,2*x+1)) FROM generate_series(1,10) AS x;

SELECT * FROM tbl_gist where c4 <@ box(point(1,1),point(10,10));

DROP TABLE tbl_gist;
