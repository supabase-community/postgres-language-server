CREATE TABLE test_tablesample (id int, name text) WITH (fillfactor=10);

INSERT INTO test_tablesample
  SELECT i, repeat(i::text, 200) FROM generate_series(0, 9) s(i);

SELECT t.id FROM test_tablesample AS t TABLESAMPLE SYSTEM (50) REPEATABLE (0);

SELECT id FROM test_tablesample TABLESAMPLE SYSTEM (100.0/11) REPEATABLE (0);

SELECT id FROM test_tablesample TABLESAMPLE SYSTEM (50) REPEATABLE (0);

SELECT id FROM test_tablesample TABLESAMPLE BERNOULLI (50) REPEATABLE (0);

SELECT id FROM test_tablesample TABLESAMPLE BERNOULLI (5.5) REPEATABLE (0);

SELECT count(*) FROM test_tablesample TABLESAMPLE SYSTEM (100);

SELECT count(*) FROM test_tablesample TABLESAMPLE SYSTEM (100) REPEATABLE (1+2);

SELECT count(*) FROM test_tablesample TABLESAMPLE SYSTEM (100) REPEATABLE (0.4);

CREATE VIEW test_tablesample_v1 AS
  SELECT id FROM test_tablesample TABLESAMPLE SYSTEM (10*2) REPEATABLE (2);

CREATE VIEW test_tablesample_v2 AS
  SELECT id FROM test_tablesample TABLESAMPLE SYSTEM (99);

BEGIN;

DECLARE tablesample_cur SCROLL CURSOR FOR
  SELECT id FROM test_tablesample TABLESAMPLE SYSTEM (50) REPEATABLE (0);

FETCH FIRST FROM tablesample_cur;

FETCH NEXT FROM tablesample_cur;

FETCH NEXT FROM tablesample_cur;

SELECT id FROM test_tablesample TABLESAMPLE SYSTEM (50) REPEATABLE (0);

FETCH NEXT FROM tablesample_cur;

FETCH NEXT FROM tablesample_cur;

FETCH NEXT FROM tablesample_cur;

FETCH FIRST FROM tablesample_cur;

FETCH NEXT FROM tablesample_cur;

FETCH NEXT FROM tablesample_cur;

FETCH NEXT FROM tablesample_cur;

FETCH NEXT FROM tablesample_cur;

FETCH NEXT FROM tablesample_cur;

CLOSE tablesample_cur;

END;

SELECT id FROM test_tablesample TABLESAMPLE SYSTEM (50) REPEATABLE (2);

SELECT * FROM test_tablesample_v1;

select count(*) from person tablesample bernoulli (100);

select count(*) from person tablesample bernoulli (100);

select count(*) from person;

SELECT count(*) FROM test_tablesample TABLESAMPLE bernoulli (('1'::text < '0'::text)::int);

select * from
  (values (0),(100)) v(pct),
  lateral (select count(*) from tenk1 tablesample bernoulli (pct)) ss;

select * from
  (values (0),(100)) v(pct),
  lateral (select count(*) from tenk1 tablesample system (pct)) ss;

select pct, count(unique1) from
  (values (0),(100)) v(pct),
  lateral (select * from tenk1 tablesample bernoulli (pct)) ss
  group by pct;

select pct, count(unique1) from
  (values (0),(100)) v(pct),
  lateral (select * from tenk1 tablesample bernoulli (pct)) ss
  group by pct;

select pct, count(unique1) from
  (values (0),(100)) v(pct),
  lateral (select * from tenk1 tablesample system (pct)) ss
  group by pct;

SELECT id FROM test_tablesample TABLESAMPLE FOOBAR (1);

SELECT id FROM test_tablesample TABLESAMPLE SYSTEM (NULL);

SELECT id FROM test_tablesample TABLESAMPLE SYSTEM (50) REPEATABLE (NULL);

SELECT id FROM test_tablesample TABLESAMPLE BERNOULLI (-1);

SELECT id FROM test_tablesample TABLESAMPLE BERNOULLI (200);

SELECT id FROM test_tablesample TABLESAMPLE SYSTEM (-1);

SELECT id FROM test_tablesample TABLESAMPLE SYSTEM (200);

SELECT id FROM test_tablesample_v1 TABLESAMPLE BERNOULLI (1);

INSERT INTO test_tablesample_v1 VALUES(1);

WITH query_select AS (SELECT * FROM test_tablesample)
SELECT * FROM query_select TABLESAMPLE BERNOULLI (5.5) REPEATABLE (1);

create table parted_sample (a int) partition by list (a);

create table parted_sample_1 partition of parted_sample for values in (1);

create table parted_sample_2 partition of parted_sample for values in (2);

select * from parted_sample tablesample bernoulli (100);

drop table parted_sample, parted_sample_1, parted_sample_2;
