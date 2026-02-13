CREATE TABLE hash_i4_heap (
	seqno 		int4,
	random 		int4
);

CREATE TABLE hash_name_heap (
	seqno 		int4,
	random 		name
);

CREATE TABLE hash_txt_heap (
	seqno 		int4,
	random 		text
);

CREATE TABLE hash_f8_heap (
	seqno		int4,
	random 		float8
);

COPY hash_i4_heap FROM 'filename';

COPY hash_name_heap FROM 'filename';

COPY hash_txt_heap FROM 'filename';

COPY hash_f8_heap FROM 'filename';

ANALYZE hash_i4_heap;

ANALYZE hash_name_heap;

ANALYZE hash_txt_heap;

ANALYZE hash_f8_heap;

CREATE INDEX hash_i4_index ON hash_i4_heap USING hash (random int4_ops);

CREATE INDEX hash_name_index ON hash_name_heap USING hash (random name_ops);

CREATE INDEX hash_txt_index ON hash_txt_heap USING hash (random text_ops);

CREATE INDEX hash_f8_index ON hash_f8_heap USING hash (random float8_ops)
  WITH (fillfactor=60);

create unique index hash_f8_index_1 on hash_f8_heap(abs(random));

create unique index hash_f8_index_2 on hash_f8_heap((seqno + 1), random);

create unique index hash_f8_index_3 on hash_f8_heap(random) where seqno > 1000;

SELECT * FROM hash_i4_heap
   WHERE hash_i4_heap.random = 843938989;

SELECT * FROM hash_i4_heap
   WHERE hash_i4_heap.random = 66766766;

SELECT * FROM hash_name_heap
   WHERE hash_name_heap.random = '1505703298'::name;

SELECT * FROM hash_name_heap
   WHERE hash_name_heap.random = '7777777'::name;

SELECT * FROM hash_txt_heap
   WHERE hash_txt_heap.random = '1351610853'::text;

SELECT * FROM hash_txt_heap
   WHERE hash_txt_heap.random = '111111112222222233333333'::text;

SELECT * FROM hash_f8_heap
   WHERE hash_f8_heap.random = '444705537'::float8;

SELECT * FROM hash_f8_heap
   WHERE hash_f8_heap.random = '88888888'::float8;

UPDATE hash_i4_heap
   SET random = 1
   WHERE hash_i4_heap.seqno = 1492;

SELECT h.seqno AS i1492, h.random AS i1
   FROM hash_i4_heap h
   WHERE h.random = 1;

UPDATE hash_i4_heap
   SET seqno = 20000
   WHERE hash_i4_heap.random = 1492795354;

SELECT h.seqno AS i20000
   FROM hash_i4_heap h
   WHERE h.random = 1492795354;

UPDATE hash_name_heap
   SET random = '0123456789abcdef'::name
   WHERE hash_name_heap.seqno = 6543;

SELECT h.seqno AS i6543, h.random AS c0_to_f
   FROM hash_name_heap h
   WHERE h.random = '0123456789abcdef'::name;

UPDATE hash_name_heap
   SET seqno = 20000
   WHERE hash_name_heap.random = '76652222'::name;

SELECT h.seqno AS emptyset
   FROM hash_name_heap h
   WHERE h.random = '76652222'::name;

UPDATE hash_txt_heap
   SET random = '0123456789abcdefghijklmnop'::text
   WHERE hash_txt_heap.seqno = 4002;

SELECT h.seqno AS i4002, h.random AS c0_to_p
   FROM hash_txt_heap h
   WHERE h.random = '0123456789abcdefghijklmnop'::text;

UPDATE hash_txt_heap
   SET seqno = 20000
   WHERE hash_txt_heap.random = '959363399'::text;

SELECT h.seqno AS t20000
   FROM hash_txt_heap h
   WHERE h.random = '959363399'::text;

UPDATE hash_f8_heap
   SET random = '-1234.1234'::float8
   WHERE hash_f8_heap.seqno = 8906;

SELECT h.seqno AS i8096, h.random AS f1234_1234
   FROM hash_f8_heap h
   WHERE h.random = '-1234.1234'::float8;

UPDATE hash_f8_heap
   SET seqno = 20000
   WHERE hash_f8_heap.random = '488912369'::float8;

SELECT h.seqno AS f20000
   FROM hash_f8_heap h
   WHERE h.random = '488912369'::float8;

CREATE TABLE hash_split_heap (keycol INT);

INSERT INTO hash_split_heap SELECT 1 FROM generate_series(1, 500) a;

CREATE INDEX hash_split_index on hash_split_heap USING HASH (keycol);

INSERT INTO hash_split_heap SELECT 1 FROM generate_series(1, 5000) a;

BEGIN;

SET enable_seqscan = OFF;

SET enable_bitmapscan = OFF;

DECLARE c CURSOR FOR SELECT * from hash_split_heap WHERE keycol = 1;

MOVE FORWARD ALL FROM c;

MOVE BACKWARD 10000 FROM c;

MOVE BACKWARD ALL FROM c;

CLOSE c;

END;

DELETE FROM hash_split_heap WHERE keycol = 1;

INSERT INTO hash_split_heap SELECT a/2 FROM generate_series(1, 25000) a;

VACUUM hash_split_heap;

ALTER INDEX hash_split_index SET (fillfactor = 10);

REINDEX INDEX hash_split_index;

DROP TABLE hash_split_heap;

CREATE TABLE hash_cleanup_heap(keycol INT);

CREATE INDEX hash_cleanup_index on hash_cleanup_heap USING HASH (keycol);

INSERT INTO hash_cleanup_heap SELECT 1 FROM generate_series(1, 500) as i;

BEGIN;

INSERT INTO hash_cleanup_heap SELECT 1 FROM generate_series(1, 1000) as i;

ROLLBACK;

CHECKPOINT;

VACUUM hash_cleanup_heap;

TRUNCATE hash_cleanup_heap;

INSERT INTO hash_cleanup_heap SELECT 1 FROM generate_series(1, 50) as i;

BEGIN;

INSERT INTO hash_cleanup_heap SELECT 1 FROM generate_series(1, 1500) as i;

ROLLBACK;

INSERT INTO hash_cleanup_heap SELECT 1 FROM generate_series(1, 500) as i;

CHECKPOINT;

VACUUM hash_cleanup_heap;

TRUNCATE hash_cleanup_heap;

INSERT INTO hash_cleanup_heap SELECT 1 FROM generate_series(1, 500) as i;

BEGIN;

INSERT INTO hash_cleanup_heap SELECT 1 FROM generate_series(1, 1500) as i;

ROLLBACK;

INSERT INTO hash_cleanup_heap SELECT 1 FROM generate_series(1, 50) as i;

CHECKPOINT;

VACUUM hash_cleanup_heap;

DROP TABLE hash_cleanup_heap;

CREATE TEMP TABLE hash_temp_heap (x int, y int);

INSERT INTO hash_temp_heap VALUES (1,1);

CREATE INDEX hash_idx ON hash_temp_heap USING hash (x);

DROP TABLE hash_temp_heap CASCADE;

CREATE TABLE hash_heap_float4 (x float4, y int);

INSERT INTO hash_heap_float4 VALUES (1.1,1);

CREATE INDEX hash_idx ON hash_heap_float4 USING hash (x);

DROP TABLE hash_heap_float4 CASCADE;

CREATE INDEX hash_f8_index2 ON hash_f8_heap USING hash (random float8_ops)
	WITH (fillfactor=9);

CREATE INDEX hash_f8_index2 ON hash_f8_heap USING hash (random float8_ops)
	WITH (fillfactor=101);
