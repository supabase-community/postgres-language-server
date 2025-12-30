CREATE INDEX onek_unique1 ON onek USING btree(unique1 int4_ops);

CREATE INDEX IF NOT EXISTS onek_unique1 ON onek USING btree(unique1 int4_ops);

CREATE INDEX onek_unique2 ON onek USING btree(unique2 int4_ops);

CREATE INDEX onek_hundred ON onek USING btree(hundred int4_ops);

CREATE INDEX onek_stringu1 ON onek USING btree(stringu1 name_ops);

CREATE INDEX tenk1_unique1 ON tenk1 USING btree(unique1 int4_ops);

CREATE INDEX tenk1_unique2 ON tenk1 USING btree(unique2 int4_ops);

CREATE INDEX tenk1_hundred ON tenk1 USING btree(hundred int4_ops);

CREATE INDEX tenk1_thous_tenthous ON tenk1 (thousand, tenthous);

CREATE INDEX tenk2_unique1 ON tenk2 USING btree(unique1 int4_ops);

CREATE INDEX tenk2_unique2 ON tenk2 USING btree(unique2 int4_ops);

CREATE INDEX tenk2_hundred ON tenk2 USING btree(hundred int4_ops);

CREATE INDEX rix ON road USING btree (name text_ops);

CREATE INDEX iix ON ihighway USING btree (name text_ops);

CREATE INDEX six ON shighway USING btree (name text_ops);

COMMENT ON INDEX six_wrong IS 'bad index';

COMMENT ON INDEX six IS 'good index';

COMMENT ON INDEX six IS NULL;

CREATE INDEX onek2_u1_prtl ON onek2 USING btree(unique1 int4_ops)
	where unique1 < 20 or unique1 > 980;

CREATE INDEX onek2_u2_prtl ON onek2 USING btree(unique2 int4_ops)
	where stringu1 < 'B';

CREATE INDEX onek2_stu1_prtl ON onek2 USING btree(stringu1 name_ops)
	where onek2.stringu1 >= 'J' and onek2.stringu1 < 'K';

CREATE TABLE slow_emp4000 (
	home_base	 box
);

CREATE TABLE fast_emp4000 (
	home_base	 box
);

COPY slow_emp4000 FROM 'filename';

INSERT INTO fast_emp4000 SELECT * FROM slow_emp4000;

ANALYZE slow_emp4000;

ANALYZE fast_emp4000;

CREATE INDEX grect2ind ON fast_emp4000 USING gist (home_base);

CREATE TEMP TABLE point_tbl AS SELECT * FROM public.point_tbl;

INSERT INTO POINT_TBL(f1) VALUES (NULL);

CREATE INDEX gpointind ON point_tbl USING gist (f1);

CREATE TEMP TABLE gpolygon_tbl AS
    SELECT polygon(home_base) AS f1 FROM slow_emp4000;

INSERT INTO gpolygon_tbl VALUES ( '(1000,0,0,1000)' );

INSERT INTO gpolygon_tbl VALUES ( '(0,1000,1000,1000)' );

CREATE TEMP TABLE gcircle_tbl AS
    SELECT circle(home_base) AS f1 FROM slow_emp4000;

CREATE INDEX ggpolygonind ON gpolygon_tbl USING gist (f1);

CREATE INDEX ggcircleind ON gcircle_tbl USING gist (f1);

SET enable_seqscan = ON;

SET enable_indexscan = OFF;

SET enable_bitmapscan = OFF;

SELECT * FROM fast_emp4000
    WHERE home_base <@ '(200,200),(2000,1000)'::box
    ORDER BY (home_base[0])[0];

SELECT count(*) FROM fast_emp4000 WHERE home_base && '(1000,1000,0,0)'::box;

SELECT count(*) FROM fast_emp4000 WHERE home_base IS NULL;

SELECT count(*) FROM gpolygon_tbl WHERE f1 && '(1000,1000,0,0)'::polygon;

SELECT count(*) FROM gcircle_tbl WHERE f1 && '<(500,500),500>'::circle;

SELECT count(*) FROM point_tbl WHERE f1 <@ box '(0,0,100,100)';

SELECT count(*) FROM point_tbl WHERE box '(0,0,100,100)' @> f1;

SELECT count(*) FROM point_tbl WHERE f1 <@ polygon '(0,0),(0,100),(100,100),(50,50),(100,0),(0,0)';

SELECT count(*) FROM point_tbl WHERE f1 <@ circle '<(50,50),50>';

SELECT count(*) FROM point_tbl p WHERE p.f1 << '(0.0, 0.0)';

SELECT count(*) FROM point_tbl p WHERE p.f1 >> '(0.0, 0.0)';

SELECT count(*) FROM point_tbl p WHERE p.f1 <<| '(0.0, 0.0)';

SELECT count(*) FROM point_tbl p WHERE p.f1 |>> '(0.0, 0.0)';

SELECT count(*) FROM point_tbl p WHERE p.f1 ~= '(-5, -12)';

SELECT * FROM point_tbl ORDER BY f1 <-> '0,1';

SELECT * FROM point_tbl WHERE f1 IS NULL;

SELECT * FROM point_tbl WHERE f1 IS NOT NULL ORDER BY f1 <-> '0,1';

SELECT * FROM point_tbl WHERE f1 <@ '(-10,-10),(10,10)':: box ORDER BY f1 <-> '0,1';

SELECT * FROM gpolygon_tbl ORDER BY f1 <-> '(0,0)'::point LIMIT 10;

SELECT circle_center(f1), round(radius(f1)) as radius FROM gcircle_tbl ORDER BY f1 <-> '(200,300)'::point LIMIT 10;

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

SELECT count(*) FROM gpolygon_tbl WHERE f1 && '(1000,1000,0,0)'::polygon;

SELECT count(*) FROM gpolygon_tbl WHERE f1 && '(1000,1000,0,0)'::polygon;

SELECT count(*) FROM gcircle_tbl WHERE f1 && '<(500,500),500>'::circle;

SELECT count(*) FROM gcircle_tbl WHERE f1 && '<(500,500),500>'::circle;

SELECT count(*) FROM point_tbl WHERE f1 <@ box '(0,0,100,100)';

SELECT count(*) FROM point_tbl WHERE f1 <@ box '(0,0,100,100)';

SELECT count(*) FROM point_tbl WHERE box '(0,0,100,100)' @> f1;

SELECT count(*) FROM point_tbl WHERE box '(0,0,100,100)' @> f1;

SELECT count(*) FROM point_tbl WHERE f1 <@ polygon '(0,0),(0,100),(100,100),(50,50),(100,0),(0,0)';

SELECT count(*) FROM point_tbl WHERE f1 <@ polygon '(0,0),(0,100),(100,100),(50,50),(100,0),(0,0)';

SELECT count(*) FROM point_tbl WHERE f1 <@ circle '<(50,50),50>';

SELECT count(*) FROM point_tbl WHERE f1 <@ circle '<(50,50),50>';

SELECT count(*) FROM point_tbl p WHERE p.f1 << '(0.0, 0.0)';

SELECT count(*) FROM point_tbl p WHERE p.f1 << '(0.0, 0.0)';

SELECT count(*) FROM point_tbl p WHERE p.f1 >> '(0.0, 0.0)';

SELECT count(*) FROM point_tbl p WHERE p.f1 >> '(0.0, 0.0)';

SELECT count(*) FROM point_tbl p WHERE p.f1 <<| '(0.0, 0.0)';

SELECT count(*) FROM point_tbl p WHERE p.f1 <<| '(0.0, 0.0)';

SELECT count(*) FROM point_tbl p WHERE p.f1 |>> '(0.0, 0.0)';

SELECT count(*) FROM point_tbl p WHERE p.f1 |>> '(0.0, 0.0)';

SELECT count(*) FROM point_tbl p WHERE p.f1 ~= '(-5, -12)';

SELECT count(*) FROM point_tbl p WHERE p.f1 ~= '(-5, -12)';

SELECT * FROM point_tbl ORDER BY f1 <-> '0,1';

SELECT * FROM point_tbl ORDER BY f1 <-> '0,1';

SELECT * FROM point_tbl WHERE f1 IS NULL;

SELECT * FROM point_tbl WHERE f1 IS NULL;

SELECT * FROM point_tbl WHERE f1 IS NOT NULL ORDER BY f1 <-> '0,1';

SELECT * FROM point_tbl WHERE f1 IS NOT NULL ORDER BY f1 <-> '0,1';

SELECT * FROM point_tbl WHERE f1 <@ '(-10,-10),(10,10)':: box ORDER BY f1 <-> '0,1';

SELECT * FROM point_tbl WHERE f1 <@ '(-10,-10),(10,10)':: box ORDER BY f1 <-> '0,1';

SELECT * FROM gpolygon_tbl ORDER BY f1 <-> '(0,0)'::point LIMIT 10;

SELECT * FROM gpolygon_tbl ORDER BY f1 <-> '(0,0)'::point LIMIT 10;

SELECT circle_center(f1), round(radius(f1)) as radius FROM gcircle_tbl ORDER BY f1 <-> '(200,300)'::point LIMIT 10;

SELECT circle_center(f1), round(radius(f1)) as radius FROM gcircle_tbl ORDER BY f1 <-> '(200,300)'::point LIMIT 10;

SELECT point(x,x), (SELECT f1 FROM gpolygon_tbl ORDER BY f1 <-> point(x,x) LIMIT 1) as c FROM generate_series(0,10,1) x;

SELECT point(x,x), (SELECT f1 FROM gpolygon_tbl ORDER BY f1 <-> point(x,x) LIMIT 1) as c FROM generate_series(0,10,1) x;

SET enable_seqscan = OFF;

SET enable_indexscan = OFF;

SET enable_bitmapscan = ON;

SELECT * FROM point_tbl WHERE f1 <@ '(-10,-10),(10,10)':: box ORDER BY f1 <-> '0,1';

SELECT * FROM point_tbl WHERE f1 <@ '(-10,-10),(10,10)':: box ORDER BY f1 <-> '0,1';

RESET enable_seqscan;

RESET enable_indexscan;

RESET enable_bitmapscan;

CREATE TABLE array_index_op_test (
	seqno		int4,
	i			int4[],
	t			text[]
);

COPY array_index_op_test FROM 'filename';

ANALYZE array_index_op_test;

SELECT * FROM array_index_op_test WHERE i = '{NULL}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i @> '{NULL}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i && '{NULL}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i <@ '{NULL}' ORDER BY seqno;

SET enable_seqscan = OFF;

SET enable_indexscan = OFF;

SET enable_bitmapscan = ON;

CREATE INDEX intarrayidx ON array_index_op_test USING gin (i);

SELECT * FROM array_index_op_test WHERE i @> '{32}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i @> '{32}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i && '{32}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i @> '{17}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i && '{17}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i @> '{32,17}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i && '{32,17}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i <@ '{38,34,32,89}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i = '{47,77}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i = '{}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i @> '{}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i && '{}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i <@ '{}' ORDER BY seqno;

CREATE INDEX textarrayidx ON array_index_op_test USING gin (t);

SELECT * FROM array_index_op_test WHERE t @> '{AAAAAAAA72908}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE t @> '{AAAAAAAA72908}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE t && '{AAAAAAAA72908}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE t @> '{AAAAAAAAAA646}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE t && '{AAAAAAAAAA646}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE t @> '{AAAAAAAA72908,AAAAAAAAAA646}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE t && '{AAAAAAAA72908,AAAAAAAAAA646}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE t <@ '{AAAAAAAA72908,AAAAAAAAAAAAAAAAAAA17075,AA88409,AAAAAAAAAAAAAAAAAA36842,AAAAAAA48038,AAAAAAAAAAAAAA10611}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE t = '{AAAAAAAAAA646,A87088}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE t = '{}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE t @> '{}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE t && '{}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE t <@ '{}' ORDER BY seqno;

DROP INDEX intarrayidx, textarrayidx;

CREATE INDEX botharrayidx ON array_index_op_test USING gin (i, t);

SELECT * FROM array_index_op_test WHERE i @> '{32}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i && '{32}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE t @> '{AAAAAAA80240}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE t && '{AAAAAAA80240}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i @> '{32}' AND t && '{AAAAAAA80240}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE i && '{32}' AND t @> '{AAAAAAA80240}' ORDER BY seqno;

SELECT * FROM array_index_op_test WHERE t = '{}' ORDER BY seqno;

RESET enable_seqscan;

RESET enable_indexscan;

RESET enable_bitmapscan;

CREATE TABLE array_gin_test (a int[]);

INSERT INTO array_gin_test SELECT ARRAY[1, g%5, g] FROM generate_series(1, 10000) g;

CREATE INDEX array_gin_test_idx ON array_gin_test USING gin (a);

SELECT COUNT(*) FROM array_gin_test WHERE a @> '{2}';

DROP TABLE array_gin_test;

CREATE INDEX gin_relopts_test ON array_index_op_test USING gin (i)
  WITH (FASTUPDATE=on, GIN_PENDING_LIST_LIMIT=128);

CREATE UNLOGGED TABLE unlogged_hash_table (id int4);

CREATE INDEX unlogged_hash_index ON unlogged_hash_table USING hash (id int4_ops);

DROP TABLE unlogged_hash_table;

SET maintenance_work_mem = '1MB';

CREATE INDEX hash_tuplesort_idx ON tenk1 USING hash (stringu1 name_ops) WITH (fillfactor = 10);

SELECT count(*) FROM tenk1 WHERE stringu1 = 'TVAAAA';

SELECT count(*) FROM tenk1 WHERE stringu1 = 'TVAAAA';

SET enable_seqscan = off;

SELECT COUNT(*) FROM tenk1 WHERE stringu1 = 'TVAAAA' OR  stringu1 = 'TVAAAB';

RESET enable_seqscan;

DROP INDEX hash_tuplesort_idx;

RESET maintenance_work_mem;

CREATE TABLE unique_tbl (i int, t text);

CREATE UNIQUE INDEX unique_idx1 ON unique_tbl (i) NULLS DISTINCT;

CREATE UNIQUE INDEX unique_idx2 ON unique_tbl (i) NULLS NOT DISTINCT;

INSERT INTO unique_tbl VALUES (1, 'one');

INSERT INTO unique_tbl VALUES (2, 'two');

INSERT INTO unique_tbl VALUES (3, 'three');

INSERT INTO unique_tbl VALUES (4, 'four');

INSERT INTO unique_tbl VALUES (5, 'one');

INSERT INTO unique_tbl (t) VALUES ('six');

INSERT INTO unique_tbl (t) VALUES ('seven');

DROP INDEX unique_idx1, unique_idx2;

INSERT INTO unique_tbl (t) VALUES ('seven');

CREATE UNIQUE INDEX unique_idx3 ON unique_tbl (i) NULLS DISTINCT;

CREATE UNIQUE INDEX unique_idx4 ON unique_tbl (i) NULLS NOT DISTINCT;

DELETE FROM unique_tbl WHERE t = 'seven';

CREATE UNIQUE INDEX unique_idx4 ON unique_tbl (i) NULLS NOT DISTINCT;

SELECT pg_get_indexdef('unique_idx3'::regclass);

SELECT pg_get_indexdef('unique_idx4'::regclass);

DROP TABLE unique_tbl;

CREATE TABLE func_index_heap (f1 text, f2 text);

CREATE UNIQUE INDEX func_index_index on func_index_heap (textcat(f1,f2));

INSERT INTO func_index_heap VALUES('ABC','DEF');

INSERT INTO func_index_heap VALUES('AB','CDEFG');

INSERT INTO func_index_heap VALUES('QWE','RTY');

INSERT INTO func_index_heap VALUES('ABCD', 'EF');

INSERT INTO func_index_heap VALUES('QWERTY');

DROP TABLE func_index_heap;

CREATE TABLE func_index_heap (f1 text, f2 text);

CREATE UNIQUE INDEX func_index_index on func_index_heap ((f1 || f2) text_ops);

INSERT INTO func_index_heap VALUES('ABC','DEF');

INSERT INTO func_index_heap VALUES('AB','CDEFG');

INSERT INTO func_index_heap VALUES('QWE','RTY');

INSERT INTO func_index_heap VALUES('ABCD', 'EF');

INSERT INTO func_index_heap VALUES('QWERTY');

create index on func_index_heap ((f1 || f2), (row(f1, f2)));

CREATE TABLE covering_index_heap (f1 int, f2 int, f3 text);

CREATE UNIQUE INDEX covering_index_index on covering_index_heap (f1,f2) INCLUDE(f3);

INSERT INTO covering_index_heap VALUES(1,1,'AAA');

INSERT INTO covering_index_heap VALUES(1,2,'AAA');

INSERT INTO covering_index_heap VALUES(1,2,'BBB');

INSERT INTO covering_index_heap VALUES(1,4,'AAA');

CREATE UNIQUE INDEX covering_pkey on covering_index_heap (f1,f2) INCLUDE(f3);

ALTER TABLE covering_index_heap ADD CONSTRAINT covering_pkey PRIMARY KEY USING INDEX
covering_pkey;

DROP TABLE covering_index_heap;

CREATE TABLE concur_heap (f1 text, f2 text);

CREATE INDEX CONCURRENTLY concur_index1 ON concur_heap(f2,f1);

CREATE INDEX CONCURRENTLY IF NOT EXISTS concur_index1 ON concur_heap(f2,f1);

INSERT INTO concur_heap VALUES  ('a','b');

INSERT INTO concur_heap VALUES  ('b','b');

CREATE UNIQUE INDEX CONCURRENTLY concur_index2 ON concur_heap(f1);

CREATE UNIQUE INDEX CONCURRENTLY IF NOT EXISTS concur_index2 ON concur_heap(f1);

INSERT INTO concur_heap VALUES ('b','x');

CREATE UNIQUE INDEX CONCURRENTLY concur_index3 ON concur_heap(f2);

CREATE INDEX CONCURRENTLY concur_index4 on concur_heap(f2) WHERE f1='a';

CREATE INDEX CONCURRENTLY concur_index5 on concur_heap(f2) WHERE f1='x';

CREATE INDEX CONCURRENTLY on concur_heap((f2||f1));

BEGIN;

CREATE INDEX CONCURRENTLY concur_index7 ON concur_heap(f1);

COMMIT;

CREATE FUNCTION predicate_stable() RETURNS bool IMMUTABLE
LANGUAGE plpgsql AS $$
BEGIN
  EXECUTE 'SELECT txid_current()';
  RETURN true;
END; $$;

CREATE INDEX CONCURRENTLY concur_index8 ON concur_heap (f1)
  WHERE predicate_stable();

DROP INDEX concur_index8;

DROP FUNCTION predicate_stable();

BEGIN;

CREATE INDEX std_index on concur_heap(f2);

COMMIT;

VACUUM FULL concur_heap;

REINDEX TABLE concur_heap;

DELETE FROM concur_heap WHERE f1 = 'b';

VACUUM FULL concur_heap;

REINDEX TABLE concur_heap;

CREATE TEMP TABLE concur_temp (f1 int, f2 text)
  ON COMMIT PRESERVE ROWS;

INSERT INTO concur_temp VALUES (1, 'foo'), (2, 'bar');

CREATE INDEX CONCURRENTLY concur_temp_ind ON concur_temp(f1);

DROP INDEX CONCURRENTLY concur_temp_ind;

DROP TABLE concur_temp;

BEGIN;

CREATE TEMP TABLE concur_temp (f1 int, f2 text)
  ON COMMIT DROP;

INSERT INTO concur_temp VALUES (1, 'foo'), (2, 'bar');

CREATE INDEX CONCURRENTLY concur_temp_ind ON concur_temp(f1);

COMMIT;

INSERT INTO concur_temp VALUES (1, 'foo'), (2, 'bar');

CREATE INDEX CONCURRENTLY concur_temp_ind ON concur_temp(f1);

DROP INDEX CONCURRENTLY concur_temp_ind;

DROP TABLE concur_temp;

DROP INDEX CONCURRENTLY "concur_index2";

DROP INDEX CONCURRENTLY IF EXISTS "concur_index2";

DROP INDEX CONCURRENTLY "concur_index2", "concur_index3";

BEGIN;

DROP INDEX CONCURRENTLY "concur_index5";

ROLLBACK;

DROP INDEX CONCURRENTLY IF EXISTS "concur_index3";

DROP INDEX CONCURRENTLY "concur_index4";

DROP INDEX CONCURRENTLY "concur_index5";

DROP INDEX CONCURRENTLY "concur_index1";

DROP INDEX CONCURRENTLY "concur_heap_expr_idx";

DROP TABLE concur_heap;

CREATE TABLE cwi_test( a int , b varchar(10), c char);

INSERT INTO cwi_test VALUES(1, 2), (3, 4), (5, 6);

CREATE UNIQUE INDEX cwi_uniq_idx ON cwi_test(a , b);

ALTER TABLE cwi_test ADD primary key USING INDEX cwi_uniq_idx;

CREATE UNIQUE INDEX cwi_uniq2_idx ON cwi_test(b , a);

ALTER TABLE cwi_test DROP CONSTRAINT cwi_uniq_idx,
	ADD CONSTRAINT cwi_replaced_pkey PRIMARY KEY
		USING INDEX cwi_uniq2_idx;

DROP INDEX cwi_replaced_pkey;

CREATE UNIQUE INDEX cwi_uniq3_idx ON cwi_test(a desc);

ALTER TABLE cwi_test ADD UNIQUE USING INDEX cwi_uniq3_idx;

CREATE UNIQUE INDEX cwi_uniq4_idx ON cwi_test(b collate "POSIX");

ALTER TABLE cwi_test ADD UNIQUE USING INDEX cwi_uniq4_idx;

DROP TABLE cwi_test;

CREATE TABLE cwi_test(a int) PARTITION BY hash (a);

create unique index on cwi_test (a);

alter table cwi_test add primary key using index cwi_test_a_idx ;

DROP TABLE cwi_test;

CREATE TABLE cwi_test(a int, b int);

CREATE UNIQUE INDEX cwi_a_nnd ON cwi_test (a) NULLS NOT DISTINCT;

ALTER TABLE cwi_test ADD PRIMARY KEY USING INDEX cwi_a_nnd;

DROP TABLE cwi_test;

CREATE TABLE syscol_table (a INT);

CREATE INDEX ON syscol_table (ctid);

CREATE INDEX ON syscol_table ((ctid >= '(1000,0)'));

CREATE INDEX ON syscol_table (a) WHERE ctid >= '(1000,0)';

DROP TABLE syscol_table;

CREATE TABLE onek_with_null AS SELECT unique1, unique2 FROM onek;

INSERT INTO onek_with_null(unique1, unique2)
VALUES (NULL, -1), (NULL, 2_147_483_647), (NULL, NULL),
       (100, NULL), (500, NULL);

CREATE UNIQUE INDEX onek_nulltest ON onek_with_null (unique2,unique1);

SET enable_seqscan = OFF;

SET enable_indexscan = ON;

SET enable_bitmapscan = ON;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL AND unique2 IS NULL;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NOT NULL;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL AND unique2 IS NOT NULL;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NOT NULL AND unique1 > 500;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL AND unique1 > 500;

SELECT unique1, unique2 FROM onek_with_null WHERE unique1 = 500 ORDER BY unique2 DESC, unique1 DESC LIMIT 1;

DROP INDEX onek_nulltest;

CREATE UNIQUE INDEX onek_nulltest ON onek_with_null (unique2 desc,unique1);

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL AND unique2 IS NULL;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NOT NULL;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL AND unique2 IS NOT NULL;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NOT NULL AND unique1 > 500;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL AND unique1 > 500;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL AND unique2 IN (-1, 0, 1);

SELECT unique1, unique2 FROM onek_with_null WHERE unique1 = 500 ORDER BY unique2 DESC, unique1 DESC LIMIT 1;

DROP INDEX onek_nulltest;

CREATE UNIQUE INDEX onek_nulltest ON onek_with_null (unique2 desc nulls last,unique1);

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL AND unique2 IS NULL;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NOT NULL;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL AND unique2 IS NOT NULL;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NOT NULL AND unique1 > 500;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL AND unique1 > 500;

SELECT unique1, unique2 FROM onek_with_null WHERE unique1 = 500 ORDER BY unique2 DESC, unique1 DESC LIMIT 1;

DROP INDEX onek_nulltest;

CREATE UNIQUE INDEX onek_nulltest ON onek_with_null (unique2  nulls first,unique1);

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL AND unique2 IS NULL;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NOT NULL;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL AND unique2 IS NOT NULL;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NOT NULL AND unique1 > 500;

SELECT count(*) FROM onek_with_null WHERE unique1 IS NULL AND unique1 > 500;

SELECT unique1, unique2 FROM onek_with_null WHERE unique1 = 500 ORDER BY unique2 DESC, unique1 DESC LIMIT 1;

DROP INDEX onek_nulltest;

CREATE UNIQUE INDEX onek_nulltest ON onek_with_null (unique2);

SET enable_seqscan = OFF;

SET enable_indexscan = ON;

SET enable_bitmapscan = OFF;

SELECT unique1, unique2 FROM onek_with_null
  ORDER BY unique2 LIMIT 2;

SELECT unique1, unique2 FROM onek_with_null WHERE unique2 >= -1
  ORDER BY unique2 LIMIT 2;

SELECT unique1, unique2 FROM onek_with_null WHERE unique2 >= 0
  ORDER BY unique2 LIMIT 2;

SELECT unique1, unique2 FROM onek_with_null
  ORDER BY unique2 DESC LIMIT 5;

SELECT unique1, unique2 FROM onek_with_null WHERE unique2 >= -1
  ORDER BY unique2 DESC LIMIT 3;

SELECT unique1, unique2 FROM onek_with_null WHERE unique2 < 999
  ORDER BY unique2 DESC LIMIT 2;

RESET enable_seqscan;

RESET enable_indexscan;

RESET enable_bitmapscan;

DROP TABLE onek_with_null;

SELECT * FROM tenk1
  WHERE thousand = 42 AND (tenthous = 1 OR tenthous = 3 OR tenthous = 42 OR tenthous = 0);

SELECT * FROM tenk1
  WHERE thousand = 42 AND (tenthous = 1 OR tenthous = 3 OR tenthous = 42 OR tenthous = 0);

SELECT * FROM tenk1
  WHERE thousand = 42 AND (tenthous = 1 OR tenthous = (SELECT 1 + 2) OR tenthous = 42);

SELECT * FROM tenk1
  WHERE thousand = 42 AND (tenthous = 1 OR tenthous = (SELECT 1 + 2) OR tenthous = 42);

SELECT * FROM tenk1
  WHERE thousand = 42 AND (tenthous = 1 OR tenthous = 3 OR tenthous = 42 OR tenthous IS NULL);

SELECT * FROM tenk1
  WHERE thousand = 42 AND (tenthous = 1::int2 OR tenthous::int2 = 3::int8 OR tenthous = 42::int8);

SELECT * FROM tenk1
  WHERE thousand = 42 AND (tenthous = 1::int2 OR tenthous::int2 = 3::int8 OR tenthous::int2 = 42::int8);

SELECT * FROM tenk1
  WHERE thousand = 42 AND (tenthous = 1::int2 OR tenthous = 3::int8 OR tenthous = 42::int8);

SELECT count(*) FROM tenk1
  WHERE hundred = 42 AND (thousand = 42 OR thousand = 99);

SELECT count(*) FROM tenk1
  WHERE hundred = 42 AND (thousand = 42 OR thousand = 99);

SELECT * FROM tenk1
  WHERE thousand = 42 AND (tenthous = 1 OR tenthous = 3 OR tenthous = 42);

SELECT * FROM tenk1
  WHERE thousand = 42 AND (tenthous = 1 OR tenthous = 3 OR tenthous = 42);

SELECT * FROM tenk1
  WHERE thousand = 42 AND (tenthous = 1::numeric OR tenthous = 3::int4 OR tenthous = 42::numeric);

SELECT * FROM tenk1
  WHERE tenthous = 1::numeric OR tenthous = 3::int4 OR tenthous = 42::numeric;

SELECT count(*) FROM tenk1 t1
  WHERE t1.thousand = 42 OR t1.thousand = (SELECT t2.tenthous FROM tenk1 t2 WHERE t2.thousand = t1.tenthous + 1 LIMIT 1);

SELECT count(*) FROM tenk1 t1
  WHERE t1.thousand = 42 OR t1.thousand = (SELECT t2.tenthous FROM tenk1 t2 WHERE t2.thousand = t1.tenthous + 1 LIMIT 1);

SELECT count(*) FROM tenk1
  WHERE hundred = 42 AND (thousand = 42 OR thousand = 99);

SELECT count(*) FROM tenk1
  WHERE hundred = 42 AND (thousand = 42 OR thousand = 99);

SELECT count(*) FROM tenk1
  WHERE hundred = 42 AND (thousand < 42 OR thousand < 99 OR 43 > thousand OR 42 > thousand);

SELECT count(*) FROM tenk1
  WHERE hundred = 42 AND (thousand < 42 OR thousand < 99 OR 43 > thousand OR 42 > thousand);

SELECT count(*) FROM tenk1
  WHERE thousand = 42 AND (tenthous = 1 OR tenthous = 3) OR thousand = 41;

SELECT count(*) FROM tenk1
  WHERE thousand = 42 AND (tenthous = 1 OR tenthous = 3) OR thousand = 41;

SELECT count(*) FROM tenk1
  WHERE hundred = 42 AND (thousand = 42 OR thousand = 99 OR tenthous < 2) OR thousand = 41;

SELECT count(*) FROM tenk1
  WHERE hundred = 42 AND (thousand = 42 OR thousand = 99 OR tenthous < 2) OR thousand = 41;

SELECT count(*) FROM tenk1
  WHERE hundred = 42 AND (thousand = 42 OR thousand = 41 OR thousand = 99 AND tenthous = 2);

SELECT count(*) FROM tenk1
  WHERE hundred = 42 AND (thousand = 42 OR thousand = 41 OR thousand = 99 AND tenthous = 2);

SELECT count(*) FROM tenk1, tenk2
  WHERE tenk1.hundred = 42 AND (tenk2.thousand = 42 OR tenk1.thousand = 41 OR tenk2.tenthous = 2) AND
  tenk2.hundred = tenk1.hundred;

SELECT count(*) FROM tenk1, tenk2
  WHERE tenk1.hundred = 42 AND (tenk2.thousand = 42 OR tenk2.thousand = 41 OR tenk2.tenthous = 2) AND
  tenk2.hundred = tenk1.hundred;

SELECT count(*) FROM tenk1 JOIN tenk2 ON
  tenk1.hundred = 42 AND (tenk2.thousand = 42 OR tenk2.thousand = 41 OR tenk2.tenthous = 2) AND
  tenk2.hundred = tenk1.hundred;

SELECT count(*) FROM tenk1 LEFT JOIN tenk2 ON
  tenk1.hundred = 42 AND (tenk2.thousand = 42 OR tenk2.thousand = 41 OR tenk2.tenthous = 2) AND
  tenk2.hundred = tenk1.hundred;

CREATE TABLE dupindexcols AS
  SELECT unique1 as id, stringu2::text as f1 FROM tenk1;

CREATE INDEX dupindexcols_i ON dupindexcols (f1, id, f1 text_pattern_ops);

ANALYZE dupindexcols;

SELECT count(*) FROM dupindexcols
    WHERE f1 BETWEEN 'WA' AND 'ZZZ' and id < 1000 and f1 ~<~ 'YX';

SELECT count(*) FROM dupindexcols
  WHERE f1 BETWEEN 'WA' AND 'ZZZ' and id < 1000 and f1 ~<~ 'YX';

SELECT unique1 FROM tenk1
WHERE unique1 IN (1,42,7)
ORDER BY unique1;

SELECT unique1 FROM tenk1
WHERE unique1 IN (1,42,7)
ORDER BY unique1;

SELECT thousand, tenthous FROM tenk1
WHERE thousand < 2 AND tenthous IN (1001,3000)
ORDER BY thousand;

SELECT thousand, tenthous FROM tenk1
WHERE thousand < 2 AND tenthous IN (1001,3000)
ORDER BY thousand;

SELECT thousand, tenthous FROM tenk1
WHERE thousand < 2 AND tenthous IN (1001,3000)
ORDER BY thousand DESC, tenthous DESC;

SELECT thousand, tenthous FROM tenk1
WHERE thousand < 2 AND tenthous IN (1001,3000)
ORDER BY thousand DESC, tenthous DESC;

SELECT thousand, tenthous FROM tenk1
WHERE thousand > 995 and tenthous in (998, 999)
ORDER BY thousand desc;

SELECT thousand, tenthous FROM tenk1
WHERE thousand > 995 and tenthous in (998, 999)
ORDER BY thousand desc;

SELECT unique1 FROM tenk1 WHERE unique1 IN (1, 42, 7) and unique1 = ANY('{7, 8, 9}');

SELECT unique1 FROM tenk1 WHERE unique1 IN (1, 42, 7) and unique1 = ANY('{7, 8, 9}');

SELECT unique1 FROM tenk1 WHERE unique1 = ANY('{7, 14, 22}') and unique1 = ANY('{33, 44}'::bigint[]);

SELECT unique1 FROM tenk1 WHERE unique1 = ANY('{7, 14, 22}') and unique1 = ANY('{33, 44}'::bigint[]);

SELECT unique1 FROM tenk1 WHERE unique1 = ANY(NULL);

SELECT unique1 FROM tenk1 WHERE unique1 = ANY(NULL);

SELECT unique1 FROM tenk1 WHERE unique1 = ANY('{NULL,NULL,NULL}');

SELECT unique1 FROM tenk1 WHERE unique1 = ANY('{NULL,NULL,NULL}');

SELECT unique1 FROM tenk1 WHERE unique1 IS NULL AND unique1 IS NULL;

SELECT unique1 FROM tenk1 WHERE unique1 IS NULL AND unique1 IS NULL;

SELECT unique1 FROM tenk1 WHERE unique1 IN (1, 42, 7) and unique1 = 1;

SELECT unique1 FROM tenk1 WHERE unique1 IN (1, 42, 7) and unique1 = 1;

SELECT unique1 FROM tenk1 WHERE unique1 IN (1, 42, 7) and unique1 = 12345;

SELECT unique1 FROM tenk1 WHERE unique1 IN (1, 42, 7) and unique1 = 12345;

SELECT unique1 FROM tenk1 WHERE unique1 IN (1, 42, 7) and unique1 >= 42;

SELECT unique1 FROM tenk1 WHERE unique1 IN (1, 42, 7) and unique1 >= 42;

SELECT unique1 FROM tenk1 WHERE unique1 IN (1, 42, 7) and unique1 > 42;

SELECT unique1 FROM tenk1 WHERE unique1 IN (1, 42, 7) and unique1 > 42;

SELECT unique1 FROM tenk1 WHERE unique1 > 9996 and unique1 >= 9999;

SELECT unique1 FROM tenk1 WHERE unique1 > 9996 and unique1 >= 9999;

SELECT unique1 FROM tenk1 WHERE unique1 < 3 and unique1 <= 3;

SELECT unique1 FROM tenk1 WHERE unique1 < 3 and unique1 <= 3;

SELECT unique1 FROM tenk1 WHERE unique1 < 3 and unique1 < (-1)::bigint;

SELECT unique1 FROM tenk1 WHERE unique1 < 3 and unique1 < (-1)::bigint;

SELECT unique1 FROM tenk1 WHERE unique1 IN (1, 42, 7) and unique1 < (-1)::bigint;

SELECT unique1 FROM tenk1 WHERE unique1 IN (1, 42, 7) and unique1 < (-1)::bigint;

SELECT unique1 FROM tenk1 WHERE (thousand, tenthous) > (NULL, 5);

SELECT unique1 FROM tenk1 WHERE (thousand, tenthous) > (NULL, 5);

SELECT thousand, tenthous FROM tenk1
WHERE thousand > -1 and thousand >= 0 AND tenthous = 3000
ORDER BY thousand;

SELECT thousand, tenthous FROM tenk1
WHERE thousand > -1 and thousand >= 0 AND tenthous = 3000
ORDER BY thousand;

SELECT thousand, tenthous FROM tenk1
WHERE thousand < 3 and thousand <= 2 AND tenthous = 1001
ORDER BY thousand;

SELECT thousand, tenthous FROM tenk1
WHERE thousand < 3 and thousand <= 2 AND tenthous = 1001
ORDER BY thousand;

SELECT thousand, tenthous FROM tenk1
WHERE thousand > -1 AND tenthous IN (1001,3000)
ORDER BY thousand limit 2;

SELECT thousand, tenthous FROM tenk1
WHERE thousand > -1 AND tenthous IN (1001,3000)
ORDER BY thousand limit 2;

select * from tenk1 where (thousand, tenthous) in ((1,1001), (null,null));

create temp table boolindex (b bool, i int, unique(b, i), junk float);

select * from boolindex order by b, i limit 10;

select * from boolindex where b order by i limit 10;

select * from boolindex where b = true order by i desc limit 10;

select * from boolindex where not b order by i limit 10;

select * from boolindex where b is true order by i desc limit 10;

select * from boolindex where b is false order by i desc limit 10;

CREATE TABLE reindex_verbose(id integer primary key);

REINDEX (VERBOSE) TABLE reindex_verbose;

DROP TABLE reindex_verbose;

CREATE TABLE concur_reindex_tab (c1 int);

REINDEX TABLE concur_reindex_tab;

REINDEX (CONCURRENTLY) TABLE concur_reindex_tab;

ALTER TABLE concur_reindex_tab ADD COLUMN c2 text;

CREATE UNIQUE INDEX concur_reindex_ind1 ON concur_reindex_tab(c1);

CREATE INDEX concur_reindex_ind2 ON concur_reindex_tab(c2);

CREATE UNIQUE INDEX concur_reindex_ind3 ON concur_reindex_tab(abs(c1));

CREATE INDEX concur_reindex_ind4 ON concur_reindex_tab(c1, c1, c2);

ALTER TABLE concur_reindex_tab ADD PRIMARY KEY USING INDEX concur_reindex_ind1;

CREATE TABLE concur_reindex_tab2 (c1 int REFERENCES concur_reindex_tab);

INSERT INTO concur_reindex_tab VALUES  (1, 'a');

INSERT INTO concur_reindex_tab VALUES  (2, 'a');

CREATE TABLE concur_reindex_tab3 (c1 int, c2 int4range, EXCLUDE USING gist (c2 WITH &&));

INSERT INTO concur_reindex_tab3 VALUES  (3, '[1,2]');

REINDEX INDEX CONCURRENTLY  concur_reindex_tab3_c2_excl;

REINDEX TABLE CONCURRENTLY concur_reindex_tab3;

INSERT INTO concur_reindex_tab3 VALUES  (4, '[2,4]');

CREATE MATERIALIZED VIEW concur_reindex_matview AS SELECT * FROM concur_reindex_tab;

SELECT pg_describe_object(classid, objid, objsubid) as obj,
       pg_describe_object(refclassid,refobjid,refobjsubid) as objref,
       deptype
FROM pg_depend
WHERE classid = 'pg_class'::regclass AND
  objid in ('concur_reindex_tab'::regclass,
            'concur_reindex_ind1'::regclass,
	    'concur_reindex_ind2'::regclass,
	    'concur_reindex_ind3'::regclass,
	    'concur_reindex_ind4'::regclass,
	    'concur_reindex_matview'::regclass)
  ORDER BY 1, 2;

REINDEX INDEX CONCURRENTLY concur_reindex_ind1;

REINDEX TABLE CONCURRENTLY concur_reindex_tab;

REINDEX TABLE CONCURRENTLY concur_reindex_matview;

SELECT pg_describe_object(classid, objid, objsubid) as obj,
       pg_describe_object(refclassid,refobjid,refobjsubid) as objref,
       deptype
FROM pg_depend
WHERE classid = 'pg_class'::regclass AND
  objid in ('concur_reindex_tab'::regclass,
            'concur_reindex_ind1'::regclass,
	    'concur_reindex_ind2'::regclass,
	    'concur_reindex_ind3'::regclass,
	    'concur_reindex_ind4'::regclass,
	    'concur_reindex_matview'::regclass)
  ORDER BY 1, 2;

CREATE TABLE testcomment (i int);

CREATE INDEX testcomment_idx1 ON testcomment (i);

COMMENT ON INDEX testcomment_idx1 IS 'test comment';

SELECT obj_description('testcomment_idx1'::regclass, 'pg_class');

REINDEX TABLE testcomment;

SELECT obj_description('testcomment_idx1'::regclass, 'pg_class');

REINDEX TABLE CONCURRENTLY testcomment ;

SELECT obj_description('testcomment_idx1'::regclass, 'pg_class');

DROP TABLE testcomment;

CREATE TABLE concur_clustered(i int);

CREATE INDEX concur_clustered_i_idx ON concur_clustered(i);

ALTER TABLE concur_clustered CLUSTER ON concur_clustered_i_idx;

REINDEX TABLE CONCURRENTLY concur_clustered;

SELECT indexrelid::regclass, indisclustered FROM pg_index
  WHERE indrelid = 'concur_clustered'::regclass;

DROP TABLE concur_clustered;

CREATE TABLE concur_replident(i int NOT NULL);

CREATE UNIQUE INDEX concur_replident_i_idx ON concur_replident(i);

ALTER TABLE concur_replident REPLICA IDENTITY
  USING INDEX concur_replident_i_idx;

SELECT indexrelid::regclass, indisreplident FROM pg_index
  WHERE indrelid = 'concur_replident'::regclass;

REINDEX TABLE CONCURRENTLY concur_replident;

SELECT indexrelid::regclass, indisreplident FROM pg_index
  WHERE indrelid = 'concur_replident'::regclass;

DROP TABLE concur_replident;

CREATE TABLE concur_appclass_tab(i tsvector, j tsvector, k tsvector);

CREATE INDEX concur_appclass_ind on concur_appclass_tab
  USING gist (i tsvector_ops (siglen='1000'), j tsvector_ops (siglen='500'));

CREATE INDEX concur_appclass_ind_2 on concur_appclass_tab
  USING gist (k tsvector_ops (siglen='300'), j tsvector_ops);

REINDEX TABLE CONCURRENTLY concur_appclass_tab;

DROP TABLE concur_appclass_tab;

CREATE TABLE concur_reindex_part (c1 int, c2 int) PARTITION BY RANGE (c1);

CREATE TABLE concur_reindex_part_0 PARTITION OF concur_reindex_part
  FOR VALUES FROM (0) TO (10) PARTITION BY list (c2);

CREATE TABLE concur_reindex_part_0_1 PARTITION OF concur_reindex_part_0
  FOR VALUES IN (1);

CREATE TABLE concur_reindex_part_0_2 PARTITION OF concur_reindex_part_0
  FOR VALUES IN (2);

CREATE TABLE concur_reindex_part_10 PARTITION OF concur_reindex_part
  FOR VALUES FROM (10) TO (20) PARTITION BY list (c2);

CREATE INDEX concur_reindex_part_index ON ONLY concur_reindex_part (c1);

CREATE INDEX concur_reindex_part_index_0 ON ONLY concur_reindex_part_0 (c1);

ALTER INDEX concur_reindex_part_index ATTACH PARTITION concur_reindex_part_index_0;

CREATE INDEX concur_reindex_part_index_10 ON ONLY concur_reindex_part_10 (c1);

ALTER INDEX concur_reindex_part_index ATTACH PARTITION concur_reindex_part_index_10;

CREATE INDEX concur_reindex_part_index_0_1 ON ONLY concur_reindex_part_0_1 (c1);

ALTER INDEX concur_reindex_part_index_0 ATTACH PARTITION concur_reindex_part_index_0_1;

CREATE INDEX concur_reindex_part_index_0_2 ON ONLY concur_reindex_part_0_2 (c1);

ALTER INDEX concur_reindex_part_index_0 ATTACH PARTITION concur_reindex_part_index_0_2;

SELECT relid, parentrelid, level FROM pg_partition_tree('concur_reindex_part_index')
  ORDER BY relid, level;

SELECT relid, parentrelid, level FROM pg_partition_tree('concur_reindex_part_index')
  ORDER BY relid, level;

SELECT pg_describe_object(classid, objid, objsubid) as obj,
       pg_describe_object(refclassid,refobjid,refobjsubid) as objref,
       deptype
FROM pg_depend
WHERE classid = 'pg_class'::regclass AND
  objid in ('concur_reindex_part'::regclass,
            'concur_reindex_part_0'::regclass,
            'concur_reindex_part_0_1'::regclass,
            'concur_reindex_part_0_2'::regclass,
            'concur_reindex_part_index'::regclass,
            'concur_reindex_part_index_0'::regclass,
            'concur_reindex_part_index_0_1'::regclass,
            'concur_reindex_part_index_0_2'::regclass)
  ORDER BY 1, 2;

REINDEX INDEX CONCURRENTLY concur_reindex_part_index_0_1;

REINDEX INDEX CONCURRENTLY concur_reindex_part_index_0_2;

SELECT relid, parentrelid, level FROM pg_partition_tree('concur_reindex_part_index')
  ORDER BY relid, level;

REINDEX TABLE CONCURRENTLY concur_reindex_part_0_1;

REINDEX TABLE CONCURRENTLY concur_reindex_part_0_2;

SELECT pg_describe_object(classid, objid, objsubid) as obj,
       pg_describe_object(refclassid,refobjid,refobjsubid) as objref,
       deptype
FROM pg_depend
WHERE classid = 'pg_class'::regclass AND
  objid in ('concur_reindex_part'::regclass,
            'concur_reindex_part_0'::regclass,
            'concur_reindex_part_0_1'::regclass,
            'concur_reindex_part_0_2'::regclass,
            'concur_reindex_part_index'::regclass,
            'concur_reindex_part_index_0'::regclass,
            'concur_reindex_part_index_0_1'::regclass,
            'concur_reindex_part_index_0_2'::regclass)
  ORDER BY 1, 2;

SELECT relid, parentrelid, level FROM pg_partition_tree('concur_reindex_part_index')
  ORDER BY relid, level;

REINDEX TABLE concur_reindex_part_index;

REINDEX TABLE CONCURRENTLY concur_reindex_part_index;

REINDEX TABLE concur_reindex_part_index_10;

REINDEX TABLE CONCURRENTLY concur_reindex_part_index_10;

BEGIN;

REINDEX INDEX concur_reindex_part_index;

ROLLBACK;

CREATE OR REPLACE FUNCTION create_relfilenode_part(relname text, indname text)
  RETURNS VOID AS
  $func$
  BEGIN
  EXECUTE format('
    CREATE TABLE %I AS
      SELECT oid, relname, relfilenode, relkind, reltoastrelid
      FROM pg_class
      WHERE oid IN
         (SELECT relid FROM pg_partition_tree(''%I''));',
	 relname, indname);
  END
  $func$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION compare_relfilenode_part(tabname text)
  RETURNS TABLE (relname name, relkind "char", state text) AS
  $func$
  BEGIN
    RETURN QUERY EXECUTE
      format(
        'SELECT  b.relname,
                 b.relkind,
                 CASE WHEN a.relfilenode = b.relfilenode THEN ''relfilenode is unchanged''
                 ELSE ''relfilenode has changed'' END
           -- Do not join with OID here as CONCURRENTLY changes it.
           FROM %I b JOIN pg_class a ON b.relname = a.relname
           ORDER BY 1;', tabname);
  END
  $func$ LANGUAGE plpgsql;

SELECT create_relfilenode_part('reindex_index_status', 'concur_reindex_part_index');

REINDEX INDEX concur_reindex_part_index;

SELECT * FROM compare_relfilenode_part('reindex_index_status');

DROP TABLE reindex_index_status;

SELECT create_relfilenode_part('reindex_index_status', 'concur_reindex_part_index');

REINDEX INDEX CONCURRENTLY concur_reindex_part_index;

SELECT * FROM compare_relfilenode_part('reindex_index_status');

DROP TABLE reindex_index_status;

REINDEX INDEX concur_reindex_part;

REINDEX INDEX CONCURRENTLY concur_reindex_part;

REINDEX INDEX concur_reindex_part_10;

REINDEX INDEX CONCURRENTLY concur_reindex_part_10;

BEGIN;

REINDEX TABLE concur_reindex_part;

ROLLBACK;

SELECT create_relfilenode_part('reindex_index_status', 'concur_reindex_part_index');

REINDEX TABLE concur_reindex_part;

SELECT * FROM compare_relfilenode_part('reindex_index_status');

DROP TABLE reindex_index_status;

SELECT create_relfilenode_part('reindex_index_status', 'concur_reindex_part_index');

REINDEX TABLE CONCURRENTLY concur_reindex_part;

SELECT * FROM compare_relfilenode_part('reindex_index_status');

DROP TABLE reindex_index_status;

DROP FUNCTION create_relfilenode_part;

DROP FUNCTION compare_relfilenode_part;

DROP TABLE concur_reindex_part;

BEGIN;

REINDEX TABLE CONCURRENTLY concur_reindex_tab;

COMMIT;

REINDEX TABLE CONCURRENTLY pg_class;

REINDEX INDEX CONCURRENTLY pg_class_oid_index;

REINDEX TABLE CONCURRENTLY pg_toast.pg_toast_1262;

REINDEX INDEX CONCURRENTLY pg_toast.pg_toast_1262_index;

REINDEX SYSTEM CONCURRENTLY postgres;

REINDEX (CONCURRENTLY) SYSTEM postgres;

REINDEX (CONCURRENTLY) SYSTEM;

REINDEX SCHEMA CONCURRENTLY pg_catalog;

REINDEX DATABASE not_current_database;

DROP MATERIALIZED VIEW concur_reindex_matview;

DROP TABLE concur_reindex_tab, concur_reindex_tab2, concur_reindex_tab3;

CREATE TABLE concur_reindex_tab4 (c1 int);

INSERT INTO concur_reindex_tab4 VALUES (1), (1), (2);

CREATE UNIQUE INDEX CONCURRENTLY concur_reindex_ind5 ON concur_reindex_tab4 (c1);

REINDEX INDEX CONCURRENTLY concur_reindex_ind5;

DROP INDEX concur_reindex_ind5_ccnew;

DELETE FROM concur_reindex_tab4 WHERE c1 = 1;

REINDEX TABLE CONCURRENTLY concur_reindex_tab4;

REINDEX INDEX CONCURRENTLY concur_reindex_ind5;

DROP TABLE concur_reindex_tab4;

CREATE TABLE concur_exprs_tab (c1 int , c2 boolean);

INSERT INTO concur_exprs_tab (c1, c2) VALUES (1369652450, FALSE),
  (414515746, TRUE),
  (897778963, FALSE);

CREATE UNIQUE INDEX concur_exprs_index_expr
  ON concur_exprs_tab ((c1::text COLLATE "C"));

CREATE UNIQUE INDEX concur_exprs_index_pred ON concur_exprs_tab (c1)
  WHERE (c1::text > 500000000::text COLLATE "C");

CREATE UNIQUE INDEX concur_exprs_index_pred_2
  ON concur_exprs_tab ((1 / c1))
  WHERE ('-H') >= (c2::TEXT) COLLATE "C";

ALTER INDEX concur_exprs_index_expr ALTER COLUMN 1 SET STATISTICS 100;

ANALYZE concur_exprs_tab;

SELECT starelid::regclass, count(*) FROM pg_statistic WHERE starelid IN (
  'concur_exprs_index_expr'::regclass,
  'concur_exprs_index_pred'::regclass,
  'concur_exprs_index_pred_2'::regclass)
  GROUP BY starelid ORDER BY starelid::regclass::text;

SELECT pg_get_indexdef('concur_exprs_index_expr'::regclass);

SELECT pg_get_indexdef('concur_exprs_index_pred'::regclass);

SELECT pg_get_indexdef('concur_exprs_index_pred_2'::regclass);

REINDEX TABLE CONCURRENTLY concur_exprs_tab;

SELECT pg_get_indexdef('concur_exprs_index_expr'::regclass);

SELECT pg_get_indexdef('concur_exprs_index_pred'::regclass);

SELECT pg_get_indexdef('concur_exprs_index_pred_2'::regclass);

ALTER TABLE concur_exprs_tab ALTER c2 TYPE TEXT;

SELECT pg_get_indexdef('concur_exprs_index_expr'::regclass);

SELECT pg_get_indexdef('concur_exprs_index_pred'::regclass);

SELECT pg_get_indexdef('concur_exprs_index_pred_2'::regclass);

SELECT starelid::regclass, count(*) FROM pg_statistic WHERE starelid IN (
  'concur_exprs_index_expr'::regclass,
  'concur_exprs_index_pred'::regclass,
  'concur_exprs_index_pred_2'::regclass)
  GROUP BY starelid ORDER BY starelid::regclass::text;

SELECT attrelid::regclass, attnum, attstattarget
  FROM pg_attribute WHERE attrelid IN (
    'concur_exprs_index_expr'::regclass,
    'concur_exprs_index_pred'::regclass,
    'concur_exprs_index_pred_2'::regclass)
  ORDER BY attrelid::regclass::text, attnum;

DROP TABLE concur_exprs_tab;

CREATE TEMP TABLE concur_temp_tab_1 (c1 int, c2 text)
  ON COMMIT PRESERVE ROWS;

INSERT INTO concur_temp_tab_1 VALUES (1, 'foo'), (2, 'bar');

CREATE INDEX concur_temp_ind_1 ON concur_temp_tab_1(c2);

REINDEX TABLE CONCURRENTLY concur_temp_tab_1;

REINDEX INDEX CONCURRENTLY concur_temp_ind_1;

BEGIN;

REINDEX INDEX CONCURRENTLY concur_temp_ind_1;

COMMIT;

CREATE INDEX concur_temp_ind_2 ON concur_temp_tab_2(c2);

REINDEX TABLE CONCURRENTLY concur_temp_tab_2;

REINDEX INDEX CONCURRENTLY concur_temp_ind_2;

BEGIN;

CREATE TEMP TABLE concur_temp_tab_3 (c1 int, c2 text)
  ON COMMIT PRESERVE ROWS;

INSERT INTO concur_temp_tab_3 VALUES (1, 'foo'), (2, 'bar');

CREATE INDEX concur_temp_ind_3 ON concur_temp_tab_3(c2);

REINDEX INDEX CONCURRENTLY concur_temp_ind_3;

COMMIT;

CREATE TABLE reindex_temp_before AS
SELECT oid, relname, relfilenode, relkind, reltoastrelid
  FROM pg_class
  WHERE relname IN ('concur_temp_ind_1', 'concur_temp_ind_2');

SELECT pg_my_temp_schema()::regnamespace as temp_schema_name ;

SELECT  b.relname,
        b.relkind,
        CASE WHEN a.relfilenode = b.relfilenode THEN 'relfilenode is unchanged'
        ELSE 'relfilenode has changed' END
  FROM reindex_temp_before b JOIN pg_class a ON b.oid = a.oid
  ORDER BY 1;

DROP TABLE concur_temp_tab_1, concur_temp_tab_2, reindex_temp_before;

SELECT * FROM tenk1 WHERE unique1 < 1 OR hundred < 2;

SELECT * FROM tenk1 WHERE unique1 < 1 OR unique1 < 3 OR hundred < 2;

CREATE TABLE bitmap_split_or (a int NOT NULL, b int NOT NULL, c int NOT NULL);

INSERT INTO bitmap_split_or (SELECT 1, 1, i FROM generate_series(1, 1000) i);

INSERT INTO bitmap_split_or (select i, 2, 2 FROM generate_series(1, 1000) i);

VACUUM ANALYZE bitmap_split_or;

CREATE INDEX t_b_partial_1_idx ON bitmap_split_or (b) WHERE a = 1;

CREATE INDEX t_b_partial_2_idx ON bitmap_split_or (b) WHERE a = 2;

SELECT * FROM bitmap_split_or WHERE (a = 1 OR a = 2) AND b = 2;

DROP INDEX t_b_partial_1_idx;

DROP INDEX t_b_partial_2_idx;

CREATE INDEX t_a_b_idx ON bitmap_split_or (a, b);

CREATE INDEX t_b_c_idx ON bitmap_split_or (b, c);

CREATE STATISTICS t_a_b_stat (mcv) ON a, b FROM bitmap_split_or;

CREATE STATISTICS t_b_c_stat (mcv) ON b, c FROM bitmap_split_or;

ANALYZE bitmap_split_or;

SELECT * FROM bitmap_split_or t1, bitmap_split_or t2
WHERE t1.a = t2.b OR t1.a = 2;

SELECT * FROM bitmap_split_or WHERE a = 1 AND (b = 1 OR b = 2) AND c = 2;

DROP TABLE bitmap_split_or;

REINDEX SCHEMA schema_to_reindex;

CREATE SCHEMA schema_to_reindex;

SET search_path = 'schema_to_reindex';

CREATE TABLE table1(col1 SERIAL PRIMARY KEY);

INSERT INTO table1 SELECT generate_series(1,400);

CREATE TABLE table2(col1 SERIAL PRIMARY KEY, col2 TEXT NOT NULL);

INSERT INTO table2 SELECT generate_series(1,400), 'abc';

CREATE INDEX ON table2(col2);

CREATE MATERIALIZED VIEW matview AS SELECT col1 FROM table2;

CREATE INDEX ON matview(col1);

CREATE VIEW view AS SELECT col2 FROM table2;

CREATE TABLE reindex_before AS
SELECT oid, relname, relfilenode, relkind, reltoastrelid
	FROM pg_class
	where relnamespace = (SELECT oid FROM pg_namespace WHERE nspname = 'schema_to_reindex');

INSERT INTO reindex_before
SELECT oid, 'pg_toast_TABLE', relfilenode, relkind, reltoastrelid
FROM pg_class WHERE oid IN
	(SELECT reltoastrelid FROM reindex_before WHERE reltoastrelid > 0);

INSERT INTO reindex_before
SELECT oid, 'pg_toast_TABLE_index', relfilenode, relkind, reltoastrelid
FROM pg_class where oid in
	(select indexrelid from pg_index where indrelid in
		(select reltoastrelid from reindex_before where reltoastrelid > 0));

REINDEX SCHEMA schema_to_reindex;

CREATE TABLE reindex_after AS SELECT oid, relname, relfilenode, relkind
	FROM pg_class
	where relnamespace = (SELECT oid FROM pg_namespace WHERE nspname = 'schema_to_reindex');

SELECT  b.relname,
        b.relkind,
        CASE WHEN a.relfilenode = b.relfilenode THEN 'relfilenode is unchanged'
        ELSE 'relfilenode has changed' END
  FROM reindex_before b JOIN pg_class a ON b.oid = a.oid
  ORDER BY 1;

REINDEX SCHEMA schema_to_reindex;

BEGIN;

REINDEX SCHEMA schema_to_reindex;

END;

REINDEX SCHEMA CONCURRENTLY schema_to_reindex;

CREATE ROLE regress_reindexuser NOLOGIN;

SET SESSION ROLE regress_reindexuser;

REINDEX SCHEMA schema_to_reindex;

RESET ROLE;

GRANT USAGE ON SCHEMA pg_toast TO regress_reindexuser;

SET SESSION ROLE regress_reindexuser;

REINDEX TABLE pg_toast.pg_toast_1262;

REINDEX INDEX pg_toast.pg_toast_1262_index;

RESET ROLE;

REVOKE USAGE ON SCHEMA pg_toast FROM regress_reindexuser;

DROP ROLE regress_reindexuser;

DROP SCHEMA schema_to_reindex CASCADE;
