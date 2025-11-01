CREATE TABLE tbl_include_reg (c1 int, c2 int, c3 int, c4 box);

INSERT INTO tbl_include_reg SELECT x, 2*x, 3*x, box('4,4,4,4') FROM generate_series(1,10) AS x;

CREATE INDEX tbl_include_reg_idx ON tbl_include_reg (c1, c2) INCLUDE (c3, c4);

CREATE INDEX ON tbl_include_reg (c1, c2) INCLUDE (c1, c3);

SELECT pg_get_indexdef(i.indexrelid)
FROM pg_index i JOIN pg_class c ON i.indexrelid = c.oid
WHERE i.indrelid = 'tbl_include_reg'::regclass ORDER BY c.relname;

CREATE TABLE tbl_include_unique1 (c1 int, c2 int, c3 int, c4 box);

INSERT INTO tbl_include_unique1 SELECT x, 2*x, 3*x, box('4,4,4,4') FROM generate_series(1,10) AS x;

CREATE UNIQUE INDEX tbl_include_unique1_idx_unique ON tbl_include_unique1 using btree (c1, c2) INCLUDE (c3, c4);

ALTER TABLE tbl_include_unique1 add UNIQUE USING INDEX tbl_include_unique1_idx_unique;

ALTER TABLE tbl_include_unique1 add UNIQUE (c1, c2) INCLUDE (c3, c4);

SELECT pg_get_indexdef(i.indexrelid)
FROM pg_index i JOIN pg_class c ON i.indexrelid = c.oid
WHERE i.indrelid = 'tbl_include_unique1'::regclass ORDER BY c.relname;

CREATE TABLE tbl_include_unique2 (c1 int, c2 int, c3 int, c4 box);

INSERT INTO tbl_include_unique2 SELECT 1, 2, 3*x, box('4,4,4,4') FROM generate_series(1,10) AS x;

CREATE UNIQUE INDEX tbl_include_unique2_idx_unique ON tbl_include_unique2 using btree (c1, c2) INCLUDE (c3, c4);

ALTER TABLE tbl_include_unique2 add UNIQUE (c1, c2) INCLUDE (c3, c4);

CREATE TABLE tbl_include_pk (c1 int, c2 int, c3 int, c4 box);

INSERT INTO tbl_include_pk SELECT 1, 2*x, 3*x, box('4,4,4,4') FROM generate_series(1,10) AS x;

ALTER TABLE tbl_include_pk add PRIMARY KEY (c1, c2) INCLUDE (c3, c4);

SELECT pg_get_indexdef(i.indexrelid)
FROM pg_index i JOIN pg_class c ON i.indexrelid = c.oid
WHERE i.indrelid = 'tbl_include_pk'::regclass ORDER BY c.relname;

CREATE TABLE tbl_include_box (c1 int, c2 int, c3 int, c4 box);

INSERT INTO tbl_include_box SELECT 1, 2*x, 3*x, box('4,4,4,4') FROM generate_series(1,10) AS x;

CREATE UNIQUE INDEX tbl_include_box_idx_unique ON tbl_include_box using btree (c1, c2) INCLUDE (c3, c4);

ALTER TABLE tbl_include_box add PRIMARY KEY USING INDEX tbl_include_box_idx_unique;

SELECT pg_get_indexdef(i.indexrelid)
FROM pg_index i JOIN pg_class c ON i.indexrelid = c.oid
WHERE i.indrelid = 'tbl_include_box'::regclass ORDER BY c.relname;

CREATE TABLE tbl_include_box_pk (c1 int, c2 int, c3 int, c4 box);

INSERT INTO tbl_include_box_pk SELECT 1, 2, 3*x, box('4,4,4,4') FROM generate_series(1,10) AS x;

ALTER TABLE tbl_include_box_pk add PRIMARY KEY (c1, c2) INCLUDE (c3, c4);

CREATE TABLE tbl (c1 int,c2 int, c3 int, c4 box,
				CONSTRAINT covering UNIQUE(c1,c2) INCLUDE(c3,c4));

SELECT indexrelid::regclass, indnatts, indnkeyatts, indisunique, indisprimary, indkey, indclass FROM pg_index WHERE indrelid = 'tbl'::regclass::oid;

SELECT pg_get_constraintdef(oid), conname, conkey FROM pg_constraint WHERE conrelid = 'tbl'::regclass::oid;

INSERT INTO tbl SELECT 1, 2, 3*x, box('4,4,4,4') FROM generate_series(1,10) AS x;

DROP TABLE tbl;

CREATE TABLE tbl (c1 int,c2 int, c3 int, c4 box,
				CONSTRAINT covering PRIMARY KEY(c1,c2) INCLUDE(c3,c4));

SELECT indexrelid::regclass, indnatts, indnkeyatts, indisunique, indisprimary, indkey, indclass FROM pg_index WHERE indrelid = 'tbl'::regclass::oid;

SELECT pg_get_constraintdef(oid), conname, conkey FROM pg_constraint WHERE conrelid = 'tbl'::regclass::oid AND contype = 'p';

INSERT INTO tbl SELECT 1, 2, 3*x, box('4,4,4,4') FROM generate_series(1,10) AS x;

INSERT INTO tbl SELECT 1, NULL, 3*x, box('4,4,4,4') FROM generate_series(1,10) AS x;

INSERT INTO tbl SELECT x, 2*x, NULL, NULL FROM generate_series(1,300) AS x;

select * from tbl where (c1,c2,c3) < (2,5,1);

select * from tbl where (c1,c2,c3) < (2,5,1);

SET enable_seqscan = off;

select * from tbl where (c1,c2,c3) < (262,1,1) limit 1;

select * from tbl where (c1,c2,c3) < (262,1,1) limit 1;

DROP TABLE tbl;

RESET enable_seqscan;

CREATE TABLE tbl (c1 int,c2 int, c3 int, c4 box,
				UNIQUE(c1,c2) INCLUDE(c3,c4));

SELECT indexrelid::regclass, indnatts, indnkeyatts, indisunique, indisprimary, indkey, indclass FROM pg_index WHERE indrelid = 'tbl'::regclass::oid;

SELECT pg_get_constraintdef(oid), conname, conkey FROM pg_constraint WHERE conrelid = 'tbl'::regclass::oid;

INSERT INTO tbl SELECT 1, 2, 3*x, box('4,4,4,4') FROM generate_series(1,10) AS x;

DROP TABLE tbl;

CREATE TABLE tbl (c1 int,c2 int, c3 int, c4 box,
				PRIMARY KEY(c1,c2) INCLUDE(c3,c4));

SELECT indexrelid::regclass, indnatts, indnkeyatts, indisunique, indisprimary, indkey, indclass FROM pg_index WHERE indrelid = 'tbl'::regclass::oid;

SELECT pg_get_constraintdef(oid), conname, conkey FROM pg_constraint WHERE conrelid = 'tbl'::regclass::oid AND contype = 'p';

INSERT INTO tbl SELECT 1, 2, 3*x, box('4,4,4,4') FROM generate_series(1,10) AS x;

INSERT INTO tbl SELECT 1, NULL, 3*x, box('4,4,4,4') FROM generate_series(1,10) AS x;

INSERT INTO tbl SELECT x, 2*x, NULL, NULL FROM generate_series(1,10) AS x;

DROP TABLE tbl;

CREATE TABLE tbl (c1 int,c2 int, c3 int, c4 box,
				EXCLUDE USING btree (c1 WITH =) INCLUDE(c3,c4));

SELECT indexrelid::regclass, indnatts, indnkeyatts, indisunique, indisprimary, indkey, indclass FROM pg_index WHERE indrelid = 'tbl'::regclass::oid;

SELECT pg_get_constraintdef(oid), conname, conkey FROM pg_constraint WHERE conrelid = 'tbl'::regclass::oid;

INSERT INTO tbl SELECT 1, 2, 3*x, box('4,4,4,4') FROM generate_series(1,10) AS x;

INSERT INTO tbl SELECT x, 2*x, NULL, NULL FROM generate_series(1,10) AS x;

DROP TABLE tbl;

CREATE TABLE tbl (c1 int,c2 int, c3 int, c4 int);

CREATE UNIQUE INDEX tbl_idx ON tbl using btree(c1, c2, c3, c4);

SELECT indexdef FROM pg_indexes WHERE tablename = 'tbl' ORDER BY indexname;

ALTER TABLE tbl DROP COLUMN c3;

SELECT indexdef FROM pg_indexes WHERE tablename = 'tbl' ORDER BY indexname;

DROP TABLE tbl;

CREATE TABLE tbl (c1 int,c2 int, c3 int, c4 box);

CREATE UNIQUE INDEX tbl_idx ON tbl using btree(c1, c2) INCLUDE(c3,c4);

SELECT indexdef FROM pg_indexes WHERE tablename = 'tbl' ORDER BY indexname;

ALTER TABLE tbl DROP COLUMN c3;

SELECT indexdef FROM pg_indexes WHERE tablename = 'tbl' ORDER BY indexname;

DROP TABLE tbl;

CREATE TABLE tbl (c1 int,c2 int, c3 int, c4 box, UNIQUE(c1, c2) INCLUDE(c3,c4));

SELECT indexdef FROM pg_indexes WHERE tablename = 'tbl' ORDER BY indexname;

ALTER TABLE tbl DROP COLUMN c3;

SELECT indexdef FROM pg_indexes WHERE tablename = 'tbl' ORDER BY indexname;

ALTER TABLE tbl DROP COLUMN c1;

SELECT indexdef FROM pg_indexes WHERE tablename = 'tbl' ORDER BY indexname;

DROP TABLE tbl;

CREATE TABLE tbl (c1 int, c2 int);

CREATE INDEX tbl_idx ON tbl (c1, (c1+0)) INCLUDE (c2);

ALTER INDEX tbl_idx ALTER COLUMN 1 SET STATISTICS 1000;

ALTER INDEX tbl_idx ALTER COLUMN 2 SET STATISTICS 1000;

ALTER INDEX tbl_idx ALTER COLUMN 3 SET STATISTICS 1000;

ALTER INDEX tbl_idx ALTER COLUMN 4 SET STATISTICS 1000;

DROP TABLE tbl;

CREATE TABLE tbl (c1 int,c2 int, c3 int, c4 box, UNIQUE(c1, c2) INCLUDE(c3,c4));

INSERT INTO tbl SELECT x, 2*x, 3*x, box('4,4,4,4') FROM generate_series(1,1000) AS x;

CREATE UNIQUE INDEX CONCURRENTLY on tbl (c1, c2) INCLUDE (c3, c4);

SELECT indexdef FROM pg_indexes WHERE tablename = 'tbl' ORDER BY indexname;

DROP TABLE tbl;

CREATE TABLE tbl (c1 int,c2 int, c3 int, c4 box, UNIQUE(c1, c2) INCLUDE(c3,c4));

SELECT indexdef FROM pg_indexes WHERE tablename = 'tbl' ORDER BY indexname;

ALTER TABLE tbl DROP COLUMN c3;

SELECT indexdef FROM pg_indexes WHERE tablename = 'tbl' ORDER BY indexname;

REINDEX INDEX tbl_c1_c2_c3_c4_key;

SELECT indexdef FROM pg_indexes WHERE tablename = 'tbl' ORDER BY indexname;

ALTER TABLE tbl DROP COLUMN c1;

SELECT indexdef FROM pg_indexes WHERE tablename = 'tbl' ORDER BY indexname;

DROP TABLE tbl;

CREATE TABLE tbl (c1 int,c2 int, c3 box, c4 box);

CREATE INDEX on tbl USING brin(c1, c2) INCLUDE (c3, c4);

CREATE INDEX on tbl USING gist(c3) INCLUDE (c1, c4);

CREATE INDEX on tbl USING spgist(c3) INCLUDE (c4);

CREATE INDEX on tbl USING gin(c1, c2) INCLUDE (c3, c4);

CREATE INDEX on tbl USING hash(c1, c2) INCLUDE (c3, c4);

CREATE INDEX on tbl USING rtree(c3) INCLUDE (c1, c4);

CREATE INDEX on tbl USING btree(c1, c2) INCLUDE (c3, c4);

DROP TABLE tbl;

CREATE TABLE tbl (c1 int, c2 int, c3 int, c4 box);

INSERT INTO tbl SELECT x, 2*x, 3*x, box('4,4,4,4') FROM generate_series(1,10) AS x;

CREATE UNIQUE INDEX tbl_idx_unique ON tbl using btree(c1, c2) INCLUDE (c3,c4);

UPDATE tbl SET c1 = 100 WHERE c1 = 2;

UPDATE tbl SET c1 = 1 WHERE c1 = 3;

UPDATE tbl SET c2 = 2 WHERE c1 = 1;

UPDATE tbl SET c3 = 1;

DELETE FROM tbl WHERE c1 = 5 OR c3 = 12;

DROP TABLE tbl;

CREATE TABLE tbl (c1 int,c2 int, c3 int, c4 box, UNIQUE(c1, c2) INCLUDE(c3,c4));

INSERT INTO tbl SELECT x, 2*x, 3*x, box('4,4,4,4') FROM generate_series(1,10) AS x;

ALTER TABLE tbl ALTER c1 TYPE bigint;

ALTER TABLE tbl ALTER c3 TYPE bigint;

DROP TABLE tbl;

CREATE TABLE nametbl (c1 int, c2 name, c3 float);

CREATE INDEX nametbl_c1_c2_idx ON nametbl (c2, c1) INCLUDE (c3);

INSERT INTO nametbl VALUES(1, 'two', 3.0);

VACUUM nametbl;

SET enable_seqscan = 0;

SELECT c2, c1, c3 FROM nametbl WHERE c2 = 'two' AND c1 = 1;

SELECT c2, c1, c3 FROM nametbl WHERE c2 = 'two' AND c1 = 1;

RESET enable_seqscan;

DROP TABLE nametbl;
