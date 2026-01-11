CREATE TABLE bt_i4_heap (
	seqno 		int4,
	random 		int4
);

CREATE TABLE bt_name_heap (
	seqno 		name,
	random 		int4
);

CREATE TABLE bt_txt_heap (
	seqno 		text,
	random 		int4
);

CREATE TABLE bt_f8_heap (
	seqno 		float8,
	random 		int4
);

COPY bt_i4_heap FROM 'filename';

COPY bt_name_heap FROM 'filename';

COPY bt_txt_heap FROM 'filename';

COPY bt_f8_heap FROM 'filename';

ANALYZE bt_i4_heap;

ANALYZE bt_name_heap;

ANALYZE bt_txt_heap;

ANALYZE bt_f8_heap;

CREATE INDEX bt_i4_index ON bt_i4_heap USING btree (seqno int4_ops);

CREATE INDEX bt_name_index ON bt_name_heap USING btree (seqno name_ops);

CREATE INDEX bt_txt_index ON bt_txt_heap USING btree (seqno text_ops);

CREATE INDEX bt_f8_index ON bt_f8_heap USING btree (seqno float8_ops);

SELECT b.*
   FROM bt_i4_heap b
   WHERE b.seqno < 1;

SELECT b.*
   FROM bt_i4_heap b
   WHERE b.seqno >= 9999;

SELECT b.*
   FROM bt_i4_heap b
   WHERE b.seqno = 4500;

SELECT b.*
   FROM bt_name_heap b
   WHERE b.seqno < '1'::name;

SELECT b.*
   FROM bt_name_heap b
   WHERE b.seqno >= '9999'::name;

SELECT b.*
   FROM bt_name_heap b
   WHERE b.seqno = '4500'::name;

SELECT b.*
   FROM bt_txt_heap b
   WHERE b.seqno < '1'::text;

SELECT b.*
   FROM bt_txt_heap b
   WHERE b.seqno >= '9999'::text;

SELECT b.*
   FROM bt_txt_heap b
   WHERE b.seqno = '4500'::text;

SELECT b.*
   FROM bt_f8_heap b
   WHERE b.seqno < '1'::float8;

SELECT b.*
   FROM bt_f8_heap b
   WHERE b.seqno >= '9999'::float8;

SELECT b.*
   FROM bt_f8_heap b
   WHERE b.seqno = '4500'::float8;

SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE (proname, pronamespace) > ('abs', 0)
ORDER BY proname, proargtypes, pronamespace LIMIT 1;

SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE (proname, pronamespace) > ('abs', 0)
ORDER BY proname, proargtypes, pronamespace LIMIT 1;

SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE (proname, pronamespace) < ('abs', 1_000_000)
ORDER BY proname DESC, proargtypes DESC, pronamespace DESC LIMIT 1;

SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE (proname, pronamespace) < ('abs', 1_000_000)
ORDER BY proname DESC, proargtypes DESC, pronamespace DESC LIMIT 1;

SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE (proname, proargtypes) >= ('abs', NULL) AND proname <= 'abs'
ORDER BY proname, proargtypes, pronamespace;

SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE (proname, proargtypes) >= ('abs', NULL) AND proname <= 'abs'
ORDER BY proname, proargtypes, pronamespace;

SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE proname >= 'abs' AND (proname, proargtypes) < ('abs', NULL)
ORDER BY proname, proargtypes, pronamespace;

SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE proname >= 'abs' AND (proname, proargtypes) < ('abs', NULL)
ORDER BY proname, proargtypes, pronamespace;

SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE proname >= 'abs' AND (proname, proargtypes) <= ('abs', NULL)
ORDER BY proname DESC, proargtypes DESC, pronamespace DESC;

SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE proname >= 'abs' AND (proname, proargtypes) <= ('abs', NULL)
ORDER BY proname DESC, proargtypes DESC, pronamespace DESC;

SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE (proname, proargtypes) > ('abs', NULL) AND proname <= 'abs'
ORDER BY proname DESC, proargtypes DESC, pronamespace DESC;

SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE (proname, proargtypes) > ('abs', NULL) AND proname <= 'abs'
ORDER BY proname DESC, proargtypes DESC, pronamespace DESC;

SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE proname = 'zzzzzz' AND (proname, proargtypes) > ('abs', NULL)
   AND pronamespace IN (1, 2, 3) AND proargtypes IN ('26 23', '5077')
ORDER BY proname, proargtypes, pronamespace;

SELECT proname, proargtypes, pronamespace
   FROM pg_proc
   WHERE proname = 'zzzzzz' AND (proname, proargtypes) > ('abs', NULL)
   AND pronamespace IN (1, 2, 3) AND proargtypes IN ('26 23', '5077')
ORDER BY proname, proargtypes, pronamespace;

SELECT thousand, tenthous
  FROM tenk1
  WHERE thousand IN (182, 183) AND tenthous > 7550;

SELECT thousand, tenthous
  FROM tenk1
  WHERE thousand IN (182, 183) AND tenthous > 7550;

set enable_seqscan to false;

set enable_indexscan to true;

set enable_bitmapscan to false;

select hundred, twenty from tenk1 where hundred < 48 order by hundred desc limit 1;

select hundred, twenty from tenk1 where hundred < 48 order by hundred desc limit 1;

select hundred, twenty from tenk1 where hundred <= 48 order by hundred desc limit 1;

select hundred, twenty from tenk1 where hundred <= 48 order by hundred desc limit 1;

select distinct hundred from tenk1 where hundred in (47, 48, 72, 82);

select distinct hundred from tenk1 where hundred in (47, 48, 72, 82);

select distinct hundred from tenk1 where hundred in (47, 48, 72, 82) order by hundred desc;

select distinct hundred from tenk1 where hundred in (47, 48, 72, 82) order by hundred desc;

select thousand from tenk1 where thousand in (364, 366,380) and tenthous = 200000;

select thousand from tenk1 where thousand in (364, 366,380) and tenthous = 200000;

set enable_seqscan to false;

set enable_indexscan to true;

set enable_bitmapscan to false;

select proname from pg_proc where proname like E'RI\\_FKey%del' order by 1;

select proname from pg_proc where proname like E'RI\\_FKey%del' order by 1;

select proname from pg_proc where proname ilike '00%foo' order by 1;

select proname from pg_proc where proname ilike '00%foo' order by 1;

select proname from pg_proc where proname ilike 'ri%foo' order by 1;

set enable_indexscan to false;

set enable_bitmapscan to true;

select proname from pg_proc where proname like E'RI\\_FKey%del' order by 1;

select proname from pg_proc where proname like E'RI\\_FKey%del' order by 1;

select proname from pg_proc where proname ilike '00%foo' order by 1;

select proname from pg_proc where proname ilike '00%foo' order by 1;

select proname from pg_proc where proname ilike 'ri%foo' order by 1;

reset enable_seqscan;

reset enable_indexscan;

reset enable_bitmapscan;

create temp table btree_bpchar (f1 text collate "C");

create index on btree_bpchar(f1 bpchar_ops) WITH (deduplicate_items=on);

insert into btree_bpchar values ('foo'), ('fool'), ('bar'), ('quux');

select * from btree_bpchar where f1 like 'foo';

select * from btree_bpchar where f1 like 'foo';

select * from btree_bpchar where f1 like 'foo%';

select * from btree_bpchar where f1 like 'foo%';

select * from btree_bpchar where f1::bpchar like 'foo';

select * from btree_bpchar where f1::bpchar like 'foo';

select * from btree_bpchar where f1::bpchar like 'foo%';

select * from btree_bpchar where f1::bpchar like 'foo%';

insert into btree_bpchar select 'foo' from generate_series(1,1500);

CREATE TABLE dedup_unique_test_table (a int) WITH (autovacuum_enabled=false);

CREATE UNIQUE INDEX dedup_unique ON dedup_unique_test_table (a) WITH (deduplicate_items=on);

CREATE UNIQUE INDEX plain_unique ON dedup_unique_test_table (a) WITH (deduplicate_items=off);

DO $$
BEGIN
    FOR r IN 1..1350 LOOP
        DELETE FROM dedup_unique_test_table;
        INSERT INTO dedup_unique_test_table SELECT 1;
    END LOOP;
END$$;

DROP INDEX plain_unique;

DELETE FROM dedup_unique_test_table WHERE a = 1;

INSERT INTO dedup_unique_test_table SELECT i FROM generate_series(0,450) i;

create table btree_tall_tbl(id int4, t text);

alter table btree_tall_tbl alter COLUMN t set storage plain;

create index btree_tall_idx on btree_tall_tbl (t, id) with (fillfactor = 10);

insert into btree_tall_tbl select g, repeat('x', 250)
from generate_series(1, 130) g;

insert into btree_tall_tbl select g, NULL
from generate_series(50, 60) g;

set enable_seqscan to false;

set enable_bitmapscan to false;

SELECT id FROM btree_tall_tbl WHERE id = 55 ORDER BY t, id;

SELECT id FROM btree_tall_tbl WHERE id = 55 ORDER BY t, id;

SELECT id FROM btree_tall_tbl WHERE id = 55 ORDER BY t DESC, id DESC;

SELECT id FROM btree_tall_tbl WHERE id = 55 ORDER BY t DESC, id DESC;

reset enable_seqscan;

reset enable_bitmapscan;

CREATE TABLE delete_test_table (a bigint, b bigint, c bigint, d bigint);

INSERT INTO delete_test_table SELECT i, 1, 2, 3 FROM generate_series(1,80000) i;

ALTER TABLE delete_test_table ADD PRIMARY KEY (a,b,c,d);

DELETE FROM delete_test_table WHERE a < 79990;

VACUUM delete_test_table;

INSERT INTO delete_test_table SELECT i, 1, 2, 3 FROM generate_series(1,1000) i;

create index on btree_tall_tbl (id int4_ops(foo=1));

CREATE TABLE btree_test_expr (n int);

CREATE FUNCTION btree_test_func() RETURNS int LANGUAGE sql IMMUTABLE RETURN 0;

BEGIN;

SET LOCAL min_parallel_table_scan_size = 0;

SET LOCAL max_parallel_maintenance_workers = 4;

CREATE INDEX btree_test_expr_idx ON btree_test_expr USING btree (btree_test_func());

COMMIT;

DROP TABLE btree_test_expr;

DROP FUNCTION btree_test_func();

CREATE INDEX btree_tall_idx2 ON btree_tall_tbl (id);

ALTER INDEX btree_tall_idx2 ALTER COLUMN id SET (n_distinct=100);

DROP INDEX btree_tall_idx2;

CREATE TABLE btree_part (id int4) PARTITION BY RANGE (id);

CREATE INDEX btree_part_idx ON btree_part(id);

ALTER INDEX btree_part_idx ALTER COLUMN id SET (n_distinct=100);

DROP TABLE btree_part;
