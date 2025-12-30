CREATE ACCESS METHOD gist2 TYPE INDEX HANDLER gisthandler;

CREATE ACCESS METHOD bogus TYPE INDEX HANDLER int4in;

CREATE ACCESS METHOD bogus TYPE INDEX HANDLER heap_tableam_handler;

CREATE INDEX grect2ind2 ON fast_emp4000 USING gist2 (home_base);

CREATE OPERATOR CLASS box_ops DEFAULT
	FOR TYPE box USING gist2 AS
	OPERATOR 1	<<,
	OPERATOR 2	&<,
	OPERATOR 3	&&,
	OPERATOR 4	&>,
	OPERATOR 5	>>,
	OPERATOR 6	~=,
	OPERATOR 7	@>,
	OPERATOR 8	<@,
	OPERATOR 9	&<|,
	OPERATOR 10	<<|,
	OPERATOR 11	|>>,
	OPERATOR 12	|&>,
	FUNCTION 1	gist_box_consistent(internal, box, smallint, oid, internal),
	FUNCTION 2	gist_box_union(internal, internal),
	-- don't need compress, decompress, or fetch functions
	FUNCTION 5	gist_box_penalty(internal, internal, internal),
	FUNCTION 6	gist_box_picksplit(internal, internal),
	FUNCTION 7	gist_box_same(box, box, internal);

CREATE INDEX grect2ind2 ON fast_emp4000 USING gist2 (home_base);

BEGIN;

DROP INDEX grect2ind;

SET enable_seqscan = OFF;

SET enable_indexscan = ON;

SET enable_bitmapscan = OFF;

SELECT * FROM fast_emp4000
    WHERE home_base <@ '(200,200),(2000,1000)'::box
    ORDER BY (home_base[0])[0];

SELECT * FROM fast_emp4000
    WHERE home_base <@ '(200,200),(2000,1000)'::box
    ORDER BY (home_base[0])[0];

SELECT count(*) FROM fast_emp4000 WHERE home_base && '(1000,1000,0,0)'::box;

SELECT count(*) FROM fast_emp4000 WHERE home_base && '(1000,1000,0,0)'::box;

SELECT count(*) FROM fast_emp4000 WHERE home_base IS NULL;

SELECT count(*) FROM fast_emp4000 WHERE home_base IS NULL;

ROLLBACK;

DROP ACCESS METHOD gist2;

BEGIN;

LOCK TABLE fast_emp4000;

DROP ACCESS METHOD gist2 CASCADE;

COMMIT;

SET default_table_access_method = '';

SET default_table_access_method = 'I do not exist AM';

SET default_table_access_method = 'btree';

CREATE ACCESS METHOD heap2 TYPE TABLE HANDLER heap_tableam_handler;

CREATE ACCESS METHOD bogus TYPE TABLE HANDLER int4in;

CREATE ACCESS METHOD bogus TYPE TABLE HANDLER bthandler;

SELECT amname, amhandler, amtype FROM pg_am where amtype = 't' ORDER BY 1, 2;

CREATE TABLE tableam_tbl_heap2(f1 int) USING heap2;

INSERT INTO tableam_tbl_heap2 VALUES(1);

SELECT f1 FROM tableam_tbl_heap2 ORDER BY f1;

CREATE TABLE tableam_tblas_heap2 USING heap2 AS SELECT * FROM tableam_tbl_heap2;

SELECT f1 FROM tableam_tbl_heap2 ORDER BY f1;

CREATE MATERIALIZED VIEW tableam_tblmv_heap2 USING heap2 AS SELECT * FROM tableam_tbl_heap2;

SELECT f1 FROM tableam_tblmv_heap2 ORDER BY f1;

CREATE TABLE tableam_parted_heap2 (a text, b int) PARTITION BY list (a) USING heap2;

SELECT a.amname FROM pg_class c, pg_am a
  WHERE c.relname = 'tableam_parted_heap2' AND a.oid = c.relam;

DROP TABLE tableam_parted_heap2;

CREATE TABLE tableam_parted_heap2 (a text, b int) PARTITION BY list (a);

SET default_table_access_method = 'heap';

CREATE TABLE tableam_parted_a_heap2 PARTITION OF tableam_parted_heap2 FOR VALUES IN ('a');

SET default_table_access_method = 'heap2';

CREATE TABLE tableam_parted_b_heap2 PARTITION OF tableam_parted_heap2 FOR VALUES IN ('b');

RESET default_table_access_method;

CREATE TABLE tableam_parted_c_heap2 PARTITION OF tableam_parted_heap2 FOR VALUES IN ('c') USING heap;

CREATE TABLE tableam_parted_d_heap2 PARTITION OF tableam_parted_heap2 FOR VALUES IN ('d') USING heap2;

SELECT
    pc.relkind,
    pa.amname,
    CASE WHEN relkind = 't' THEN
        (SELECT 'toast for ' || relname::regclass FROM pg_class pcm WHERE pcm.reltoastrelid = pc.oid)
    ELSE
        relname::regclass::text
    END COLLATE "C" AS relname
FROM pg_class AS pc,
    pg_am AS pa
WHERE pa.oid = pc.relam
   AND pa.amname = 'heap2'
ORDER BY 3, 1, 2;

SELECT pg_describe_object(classid,objid,objsubid) AS obj
FROM pg_depend, pg_am
WHERE pg_depend.refclassid = 'pg_am'::regclass
    AND pg_am.oid = pg_depend.refobjid
    AND pg_am.amname = 'heap2'
ORDER BY classid, objid, objsubid;

CREATE TABLE heaptable USING heap AS
  SELECT a, repeat(a::text, 100) FROM generate_series(1,9) AS a;

SELECT amname FROM pg_class c, pg_am am
  WHERE c.relam = am.oid AND c.oid = 'heaptable'::regclass;

ALTER TABLE heaptable SET ACCESS METHOD heap2;

SELECT pg_describe_object(classid, objid, objsubid) as obj,
       pg_describe_object(refclassid, refobjid, refobjsubid) as objref,
       deptype
  FROM pg_depend
  WHERE classid = 'pg_class'::regclass AND
        objid = 'heaptable'::regclass
  ORDER BY 1, 2;

ALTER TABLE heaptable SET ACCESS METHOD heap;

SELECT pg_describe_object(classid, objid, objsubid) as obj,
       pg_describe_object(refclassid, refobjid, refobjsubid) as objref,
       deptype
  FROM pg_depend
  WHERE classid = 'pg_class'::regclass AND
        objid = 'heaptable'::regclass
  ORDER BY 1, 2;

ALTER TABLE heaptable SET ACCESS METHOD heap2;

SELECT amname FROM pg_class c, pg_am am
  WHERE c.relam = am.oid AND c.oid = 'heaptable'::regclass;

SELECT COUNT(a), COUNT(1) FILTER(WHERE a=1) FROM heaptable;

BEGIN;

SET LOCAL default_table_access_method TO heap2;

ALTER TABLE heaptable SET ACCESS METHOD DEFAULT;

SELECT amname FROM pg_class c, pg_am am
  WHERE c.relam = am.oid AND c.oid = 'heaptable'::regclass;

SET LOCAL default_table_access_method TO heap;

ALTER TABLE heaptable SET ACCESS METHOD DEFAULT;

SELECT amname FROM pg_class c, pg_am am
  WHERE c.relam = am.oid AND c.oid = 'heaptable'::regclass;

ROLLBACK;

CREATE MATERIALIZED VIEW heapmv USING heap AS SELECT * FROM heaptable;

SELECT amname FROM pg_class c, pg_am am
  WHERE c.relam = am.oid AND c.oid = 'heapmv'::regclass;

ALTER MATERIALIZED VIEW heapmv SET ACCESS METHOD heap2;

SELECT amname FROM pg_class c, pg_am am
  WHERE c.relam = am.oid AND c.oid = 'heapmv'::regclass;

SELECT COUNT(a), COUNT(1) FILTER(WHERE a=1) FROM heapmv;

ALTER TABLE heaptable SET ACCESS METHOD heap, SET ACCESS METHOD heap2;

ALTER TABLE heaptable SET ACCESS METHOD DEFAULT, SET ACCESS METHOD heap2;

ALTER MATERIALIZED VIEW heapmv SET ACCESS METHOD heap, SET ACCESS METHOD heap2;

DROP MATERIALIZED VIEW heapmv;

DROP TABLE heaptable;

CREATE TABLE am_partitioned(x INT, y INT) PARTITION BY hash (x) USING heap2;

SELECT pg_describe_object(classid, objid, objsubid) AS obj,
       pg_describe_object(refclassid, refobjid, refobjsubid) as refobj
  FROM pg_depend, pg_am
  WHERE pg_depend.refclassid = 'pg_am'::regclass
    AND pg_am.oid = pg_depend.refobjid
    AND pg_depend.objid = 'am_partitioned'::regclass;

DROP TABLE am_partitioned;

BEGIN;

SET LOCAL default_table_access_method = 'heap';

CREATE TABLE am_partitioned(x INT, y INT) PARTITION BY hash (x);

SELECT relam FROM pg_class WHERE relname = 'am_partitioned';

SELECT pg_describe_object(classid, objid, objsubid) AS obj,
       pg_describe_object(refclassid, refobjid, refobjsubid) as refobj
  FROM pg_depend, pg_am
  WHERE pg_depend.refclassid = 'pg_am'::regclass
    AND pg_am.oid = pg_depend.refobjid
    AND pg_depend.objid = 'am_partitioned'::regclass;

ALTER TABLE am_partitioned SET ACCESS METHOD heap2;

SELECT a.amname FROM pg_class c, pg_am a
  WHERE c.relname = 'am_partitioned' AND a.oid = c.relam;

SELECT pg_describe_object(classid, objid, objsubid) AS obj,
       pg_describe_object(refclassid, refobjid, refobjsubid) as refobj
  FROM pg_depend, pg_am
  WHERE pg_depend.refclassid = 'pg_am'::regclass
    AND pg_am.oid = pg_depend.refobjid
    AND pg_depend.objid = 'am_partitioned'::regclass;

SET LOCAL default_table_access_method = 'heap2';

ALTER TABLE am_partitioned SET ACCESS METHOD heap;

SELECT a.amname FROM pg_class c, pg_am a
  WHERE c.relname = 'am_partitioned' AND a.oid = c.relam;

SELECT pg_describe_object(classid, objid, objsubid) AS obj,
       pg_describe_object(refclassid, refobjid, refobjsubid) as refobj
  FROM pg_depend, pg_am
  WHERE pg_depend.refclassid = 'pg_am'::regclass
    AND pg_am.oid = pg_depend.refobjid
    AND pg_depend.objid = 'am_partitioned'::regclass;

SET LOCAL default_table_access_method = 'heap2';

ALTER TABLE am_partitioned SET ACCESS METHOD heap2;

SELECT a.amname FROM pg_class c, pg_am a
  WHERE c.relname = 'am_partitioned' AND a.oid = c.relam;

ALTER TABLE am_partitioned SET ACCESS METHOD DEFAULT;

SELECT relam FROM pg_class WHERE relname = 'am_partitioned';

SELECT relam FROM pg_class WHERE relname = 'am_partitioned';

SET LOCAL default_table_access_method = 'heap';

CREATE TABLE am_partitioned_0 PARTITION OF am_partitioned
  FOR VALUES WITH (MODULUS 10, REMAINDER 0);

SET LOCAL default_table_access_method = 'heap2';

CREATE TABLE am_partitioned_1 PARTITION OF am_partitioned
  FOR VALUES WITH (MODULUS 10, REMAINDER 1);

SET LOCAL default_table_access_method = 'heap';

ALTER TABLE am_partitioned SET ACCESS METHOD heap2;

CREATE TABLE am_partitioned_2 PARTITION OF am_partitioned
  FOR VALUES WITH (MODULUS 10, REMAINDER 2);

ALTER TABLE am_partitioned SET ACCESS METHOD DEFAULT;

SELECT relam FROM pg_class WHERE relname = 'am_partitioned';

CREATE TABLE am_partitioned_3 PARTITION OF am_partitioned
  FOR VALUES WITH (MODULUS 10, REMAINDER 3);

ALTER TABLE am_partitioned SET ACCESS METHOD DEFAULT;

CREATE TABLE am_partitioned_5p PARTITION OF am_partitioned
  FOR VALUES WITH (MODULUS 10, REMAINDER 5) PARTITION BY hash(y);

CREATE TABLE am_partitioned_5p1 PARTITION OF am_partitioned_5p
  FOR VALUES WITH (MODULUS 10, REMAINDER 1);

ALTER TABLE am_partitioned SET ACCESS METHOD heap2;

CREATE TABLE am_partitioned_6p PARTITION OF am_partitioned
  FOR VALUES WITH (MODULUS 10, REMAINDER 6) PARTITION BY hash(y);

CREATE TABLE am_partitioned_6p1 PARTITION OF am_partitioned_6p
  FOR VALUES WITH (MODULUS 10, REMAINDER 1);

SELECT c.relname, a.amname FROM pg_class c, pg_am a
  WHERE c.relam = a.oid AND
        c.relname LIKE 'am_partitioned%'
UNION ALL
SELECT c.relname, 'default' FROM pg_class c
  WHERE c.relam = 0
        AND c.relname LIKE 'am_partitioned%' ORDER BY 1;

DROP TABLE am_partitioned;

COMMIT;

BEGIN;

SET LOCAL default_table_access_method = 'heap2';

CREATE TABLE tableam_tbl_heapx(f1 int);

CREATE TABLE tableam_tblas_heapx AS SELECT * FROM tableam_tbl_heapx;

SELECT INTO tableam_tblselectinto_heapx FROM tableam_tbl_heapx;

CREATE MATERIALIZED VIEW tableam_tblmv_heapx USING heap2 AS SELECT * FROM tableam_tbl_heapx;

CREATE TABLE tableam_parted_heapx (a text, b int) PARTITION BY list (a);

CREATE TABLE tableam_parted_1_heapx PARTITION OF tableam_parted_heapx FOR VALUES IN ('a', 'b');

CREATE TABLE tableam_parted_2_heapx PARTITION OF tableam_parted_heapx FOR VALUES IN ('c', 'd') USING heap;

CREATE VIEW tableam_view_heapx AS SELECT * FROM tableam_tbl_heapx;

CREATE SEQUENCE tableam_seq_heapx;

CREATE FOREIGN DATA WRAPPER fdw_heap2 VALIDATOR postgresql_fdw_validator;

CREATE SERVER fs_heap2 FOREIGN DATA WRAPPER fdw_heap2 ;

CREATE FOREIGN table tableam_fdw_heapx () SERVER fs_heap2;

SELECT
    pc.relkind,
    pa.amname,
    CASE WHEN relkind = 't' THEN
        (SELECT 'toast for ' || relname::regclass FROM pg_class pcm WHERE pcm.reltoastrelid = pc.oid)
    ELSE
        relname::regclass::text
    END COLLATE "C" AS relname
FROM pg_class AS pc
    LEFT JOIN pg_am AS pa ON (pa.oid = pc.relam)
WHERE pc.relname LIKE 'tableam_%_heapx'
ORDER BY 3, 1, 2;

ROLLBACK;

CREATE TABLE i_am_a_failure() USING i_do_not_exist_am;

CREATE TABLE i_am_a_failure() USING "I do not exist AM";

CREATE TABLE i_am_a_failure() USING "btree";

CREATE FOREIGN TABLE fp PARTITION OF tableam_parted_a_heap2 DEFAULT SERVER x;

DROP ACCESS METHOD heap2;
