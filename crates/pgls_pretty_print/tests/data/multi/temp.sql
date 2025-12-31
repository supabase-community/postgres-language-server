CREATE TABLE temptest(col int);

CREATE INDEX i_temptest ON temptest(col);

CREATE TEMP TABLE temptest(tcol int);

CREATE INDEX i_temptest ON temptest(tcol);

SELECT * FROM temptest;

DROP INDEX i_temptest;

DROP TABLE temptest;

SELECT * FROM temptest;

DROP INDEX i_temptest;

DROP TABLE temptest;

CREATE TABLE temptest(col int);

INSERT INTO temptest VALUES (1);

CREATE TEMP TABLE temptest(tcol float);

INSERT INTO temptest VALUES (2.1);

SELECT * FROM temptest;

DROP TABLE temptest;

SELECT * FROM temptest;

DROP TABLE temptest;

CREATE TEMP TABLE temptest(col int);

SELECT * FROM temptest;

CREATE INDEX ON temptest(bit_length(''));

BEGIN;

INSERT INTO temptest VALUES (1);

INSERT INTO temptest VALUES (2);

SELECT * FROM temptest;

COMMIT;

SELECT * FROM temptest;

DROP TABLE temptest;

BEGIN;

SELECT * FROM temptest;

COMMIT;

SELECT * FROM temptest;

DROP TABLE temptest;

BEGIN;

CREATE TEMP TABLE temptest(col int) ON COMMIT DROP;

INSERT INTO temptest VALUES (1);

INSERT INTO temptest VALUES (2);

SELECT * FROM temptest;

COMMIT;

SELECT * FROM temptest;

BEGIN;

CREATE TEMP TABLE temptest(col) ON COMMIT DROP AS SELECT 1;

SELECT * FROM temptest;

COMMIT;

SELECT * FROM temptest;

BEGIN;

do $$
begin
  execute format($cmd$
    CREATE TEMP TABLE temptest (col text CHECK (col < %L)) ON COMMIT DROP
  $cmd$,
    (SELECT string_agg(g.i::text || ':' || random()::text, '|')
     FROM generate_series(1, 100) g(i)));
end$$;

SELECT * FROM temptest;

COMMIT;

SELECT * FROM temptest;

BEGIN;

CREATE TEMP TABLE temptest1(col int PRIMARY KEY);

INSERT INTO temptest1 VALUES (1);

INSERT INTO temptest2 VALUES (1);

COMMIT;

SELECT * FROM temptest1;

SELECT * FROM temptest2;

BEGIN;

CREATE TEMP TABLE temptest4(col int REFERENCES temptest3);

COMMIT;

create table public.whereami (f1 text);

insert into public.whereami values ('public');

create temp table whereami (f1 text);

insert into whereami values ('temp');

create function public.whoami() returns text
  as $$select 'public'::text$$ language sql;

create function pg_temp.whoami() returns text
  as $$select 'temp'::text$$ language sql;

select * from whereami;

select whoami();

set search_path = pg_temp, public;

select * from whereami;

select whoami();

set search_path = public, pg_temp;

select * from whereami;

select whoami();

select pg_temp.whoami();

drop table public.whereami;

set search_path = pg_temp, public;

create domain pg_temp.nonempty as text check (value <> '');

select nonempty('');

select pg_temp.nonempty('');

select ''::nonempty;

reset search_path;

begin;

insert into temp_parted_oncommit values (1);

commit;

select * from temp_parted_oncommit;

drop table temp_parted_oncommit;

begin;

create temp table temp_parted_oncommit_test (a int)
  partition by list (a) on commit drop;

create temp table temp_parted_oncommit_test2
  partition of temp_parted_oncommit_test
  for values in (2) on commit drop;

insert into temp_parted_oncommit_test values (1), (2);

commit;

select relname from pg_class where relname ~ '^temp_parted_oncommit_test';

begin;

create temp table temp_parted_oncommit_test1
  partition of temp_parted_oncommit_test
  for values in (1) on commit preserve rows;

create temp table temp_parted_oncommit_test2
  partition of temp_parted_oncommit_test
  for values in (2) on commit drop;

insert into temp_parted_oncommit_test values (1), (2);

commit;

select * from temp_parted_oncommit_test;

select relname from pg_class where relname ~ '^temp_parted_oncommit_test'
  order by relname;

drop table temp_parted_oncommit_test;

begin;

create temp table temp_inh_oncommit_test (a int) on commit drop;

insert into temp_inh_oncommit_test1 values (1);

commit;

select relname from pg_class where relname ~ '^temp_inh_oncommit_test';

begin;

create temp table temp_inh_oncommit_test1 ()
  inherits(temp_inh_oncommit_test) on commit drop;

insert into temp_inh_oncommit_test1 values (1);

insert into temp_inh_oncommit_test values (1);

commit;

select * from temp_inh_oncommit_test;

select relname from pg_class where relname ~ '^temp_inh_oncommit_test';

drop table temp_inh_oncommit_test;

begin;

create function pg_temp.twophase_func() returns void as
  $$ select '2pc_func'::text $$ language sql;

prepare transaction 'twophase_func';

create function pg_temp.twophase_func() returns void as
  $$ select '2pc_func'::text $$ language sql;

begin;

drop function pg_temp.twophase_func();

prepare transaction 'twophase_func';

begin;

create operator pg_temp.@@ (leftarg = int4, rightarg = int4, procedure = int4mi);

prepare transaction 'twophase_operator';

begin;

create type pg_temp.twophase_type as (a int);

prepare transaction 'twophase_type';

begin;

create view pg_temp.twophase_view as select 1;

prepare transaction 'twophase_view';

begin;

create sequence pg_temp.twophase_seq;

prepare transaction 'twophase_sequence';

create temp table twophase_tab (a int);

begin;

select a from twophase_tab;

prepare transaction 'twophase_tab';

begin;

insert into twophase_tab values (1);

prepare transaction 'twophase_tab';

begin;

lock twophase_tab in access exclusive mode;

prepare transaction 'twophase_tab';

begin;

drop table twophase_tab;

prepare transaction 'twophase_tab';

SET search_path TO 'pg_temp';

BEGIN;

SELECT current_schema() ~ 'pg_temp' AS is_temp_schema;

PREPARE TRANSACTION 'twophase_search';

SET temp_buffers = 100;

CREATE TEMPORARY TABLE test_temp(a int not null unique, b TEXT not null, cnt int not null);

INSERT INTO test_temp SELECT generate_series(1, 10000) as id, repeat('a', 200), 0;

SELECT pg_relation_size('test_temp') / current_setting('block_size')::int8 > 200;

CREATE FUNCTION test_temp_pin(p_start int, p_end int)
RETURNS void
LANGUAGE plpgsql
AS $f$
  DECLARE
      cursorname text;
      query text;
  BEGIN
    FOR i IN p_start..p_end LOOP
       cursorname = 'c_'||i;
       query = format($q$DECLARE %I CURSOR FOR SELECT ctid FROM test_temp WHERE ctid >= '( %s, 1)'::tid $q$, cursorname, i);
       EXECUTE query;
       EXECUTE 'FETCH NEXT FROM '||cursorname;
       -- for test development
       -- RAISE NOTICE '%: %', cursorname, query;
    END LOOP;
  END;
$f$;

BEGIN;

SELECT test_temp_pin(0, 9);

SELECT test_temp_pin(10, 105);

ROLLBACK;

BEGIN;

SELECT test_temp_pin(0, 9);

FETCH NEXT FROM c_3;

SAVEPOINT rescue_me;

SELECT test_temp_pin(10, 105);

ROLLBACK TO SAVEPOINT rescue_me;

FETCH NEXT FROM c_3;

SELECT test_temp_pin(10, 94);

SELECT count(*), max(a) max_a, min(a) min_a, max(cnt) max_cnt FROM test_temp;

ROLLBACK;

BEGIN;

SELECT test_temp_pin(0, 1);

DROP TABLE test_temp;

COMMIT;

BEGIN;

SELECT test_temp_pin(0, 1);

TRUNCATE test_temp;

COMMIT;

SELECT count(*), max(a) max_a, min(a) min_a, max(cnt) max_cnt FROM test_temp;

INSERT INTO test_temp(a, b, cnt) VALUES (-1, '', 0);

BEGIN;

INSERT INTO test_temp(a, b, cnt) VALUES (-2, '', 0);

DROP TABLE test_temp;

ROLLBACK;

SELECT count(*), max(a) max_a, min(a) min_a, max(cnt) max_cnt FROM test_temp;

UPDATE test_temp SET cnt = cnt + 1 WHERE a = -1;

BEGIN;

DROP TABLE test_temp;

ROLLBACK;

SELECT count(*), max(a) max_a, min(a) min_a, max(cnt) max_cnt FROM test_temp;

UPDATE test_temp SET cnt = cnt + 1 WHERE a = -1;

BEGIN;

TRUNCATE test_temp;

ROLLBACK;

SELECT count(*), max(a) max_a, min(a) min_a, max(cnt) max_cnt FROM test_temp;

DROP FUNCTION test_temp_pin(int, int);
