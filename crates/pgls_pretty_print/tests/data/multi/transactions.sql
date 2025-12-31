BEGIN;

CREATE TABLE xacttest (a smallint, b real);

INSERT INTO xacttest VALUES
  (56, 7.8),
  (100, 99.097),
  (0, 0.09561),
  (42, 324.78);

INSERT INTO xacttest (a, b) VALUES (777, 777.777);

END;

SELECT a FROM xacttest WHERE a > 100;

BEGIN;

CREATE TABLE disappear (a int4);

DELETE FROM xacttest;

SELECT * FROM xacttest;

ABORT;

SELECT oid FROM pg_class WHERE relname = 'disappear';

SELECT * FROM xacttest;

BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE;

SELECT COUNT(*) FROM xacttest;

RESET transaction_isolation;

END;

BEGIN TRANSACTION READ ONLY;

SELECT COUNT(*) FROM xacttest;

RESET transaction_read_only;

END;

BEGIN TRANSACTION DEFERRABLE;

SELECT COUNT(*) FROM xacttest;

RESET transaction_deferrable;

END;

CREATE FUNCTION errfunc() RETURNS int LANGUAGE SQL AS 'SELECT 1'
SET transaction_read_only = on;

CREATE TABLE writetest (a int);

CREATE TEMPORARY TABLE temptest (a int);

BEGIN;

SET TRANSACTION ISOLATION LEVEL SERIALIZABLE, READ ONLY, DEFERRABLE;

SELECT * FROM writetest;

SET TRANSACTION READ WRITE;

COMMIT;

BEGIN;

SET TRANSACTION READ ONLY;

SET TRANSACTION READ WRITE;

SET TRANSACTION READ ONLY;

SELECT * FROM writetest;

SAVEPOINT x;

SET TRANSACTION READ ONLY;

SELECT * FROM writetest;

SET TRANSACTION READ ONLY;

SET TRANSACTION READ WRITE;

COMMIT;

BEGIN;

SET TRANSACTION READ WRITE;

SAVEPOINT x;

SET TRANSACTION READ WRITE;

SET TRANSACTION READ ONLY;

SELECT * FROM writetest;

SET TRANSACTION READ ONLY;

SET TRANSACTION READ WRITE;

COMMIT;

BEGIN;

SET TRANSACTION READ WRITE;

SAVEPOINT x;

SET TRANSACTION READ ONLY;

SELECT * FROM writetest;

ROLLBACK TO SAVEPOINT x;

SHOW transaction_read_only;

SAVEPOINT y;

SET TRANSACTION READ ONLY;

SELECT * FROM writetest;

RELEASE SAVEPOINT y;

SHOW transaction_read_only;

COMMIT;

SET SESSION CHARACTERISTICS AS TRANSACTION READ ONLY;

DROP TABLE writetest;

INSERT INTO writetest VALUES (1);

SELECT * FROM writetest;

DELETE FROM temptest;

UPDATE temptest SET a = 0 FROM writetest WHERE temptest.a = 1 AND writetest.a = temptest.a;

UPDATE writetest SET a = 0;

EXECUTE test;

SELECT * FROM writetest, temptest;

CREATE TABLE test AS SELECT * FROM writetest;

START TRANSACTION READ WRITE;

DROP TABLE writetest;

COMMIT;

SET SESSION CHARACTERISTICS AS TRANSACTION READ WRITE;

CREATE TABLE trans_foobar (a int);

BEGIN;

CREATE TABLE trans_foo (a int);

SAVEPOINT one;

DROP TABLE trans_foo;

CREATE TABLE trans_bar (a int);

ROLLBACK TO SAVEPOINT one;

RELEASE SAVEPOINT one;

SAVEPOINT two;

CREATE TABLE trans_baz (a int);

RELEASE SAVEPOINT two;

drop TABLE trans_foobar;

CREATE TABLE trans_barbaz (a int);

COMMIT;

SELECT * FROM trans_foo;

SELECT * FROM trans_bar;

SELECT * FROM trans_barbaz;

SELECT * FROM trans_baz;

BEGIN;

INSERT INTO trans_foo VALUES (1);

SAVEPOINT one;

INSERT into trans_bar VALUES (1);

ROLLBACK TO one;

RELEASE SAVEPOINT one;

SAVEPOINT two;

INSERT into trans_barbaz VALUES (1);

RELEASE two;

SAVEPOINT three;

SAVEPOINT four;

INSERT INTO trans_foo VALUES (2);

RELEASE SAVEPOINT four;

ROLLBACK TO SAVEPOINT three;

RELEASE SAVEPOINT three;

INSERT INTO trans_foo VALUES (3);

COMMIT;

SELECT * FROM trans_foo;

SELECT * FROM trans_barbaz;

BEGIN;

SAVEPOINT one;

SELECT trans_foo;

ROLLBACK TO SAVEPOINT one;

RELEASE SAVEPOINT one;

SAVEPOINT two;

CREATE TABLE savepoints (a int);

SAVEPOINT three;

INSERT INTO savepoints VALUES (1);

SAVEPOINT four;

INSERT INTO savepoints VALUES (2);

SAVEPOINT five;

INSERT INTO savepoints VALUES (3);

ROLLBACK TO SAVEPOINT five;

COMMIT;

COMMIT;

SELECT * FROM savepoints;

BEGIN;

SAVEPOINT one;

DELETE FROM savepoints WHERE a=1;

RELEASE SAVEPOINT one;

SAVEPOINT two;

DELETE FROM savepoints WHERE a=1;

SAVEPOINT three;

DELETE FROM savepoints WHERE a=2;

ROLLBACK;

COMMIT;

SELECT * FROM savepoints;

BEGIN;

INSERT INTO savepoints VALUES (4);

SAVEPOINT one;

INSERT INTO savepoints VALUES (5);

SELECT trans_foo;

COMMIT;

SELECT * FROM savepoints;

BEGIN;

INSERT INTO savepoints VALUES (6);

SAVEPOINT one;

INSERT INTO savepoints VALUES (7);

RELEASE SAVEPOINT one;

INSERT INTO savepoints VALUES (8);

COMMIT;

SELECT a.xmin = b.xmin FROM savepoints a, savepoints b WHERE a.a=6 AND b.a=8;

SELECT a.xmin = b.xmin FROM savepoints a, savepoints b WHERE a.a=6 AND b.a=7;

BEGIN;

INSERT INTO savepoints VALUES (9);

SAVEPOINT one;

INSERT INTO savepoints VALUES (10);

ROLLBACK TO SAVEPOINT one;

INSERT INTO savepoints VALUES (11);

COMMIT;

SELECT a FROM savepoints WHERE a in (9, 10, 11);

SELECT a.xmin = b.xmin FROM savepoints a, savepoints b WHERE a.a=9 AND b.a=11;

BEGIN;

INSERT INTO savepoints VALUES (12);

SAVEPOINT one;

INSERT INTO savepoints VALUES (13);

SAVEPOINT two;

INSERT INTO savepoints VALUES (14);

ROLLBACK TO SAVEPOINT one;

INSERT INTO savepoints VALUES (15);

SAVEPOINT two;

INSERT INTO savepoints VALUES (16);

SAVEPOINT three;

INSERT INTO savepoints VALUES (17);

COMMIT;

SELECT a FROM savepoints WHERE a BETWEEN 12 AND 17;

BEGIN;

INSERT INTO savepoints VALUES (18);

SAVEPOINT one;

INSERT INTO savepoints VALUES (19);

SAVEPOINT two;

INSERT INTO savepoints VALUES (20);

ROLLBACK TO SAVEPOINT one;

INSERT INTO savepoints VALUES (21);

ROLLBACK TO SAVEPOINT one;

INSERT INTO savepoints VALUES (22);

COMMIT;

SELECT a FROM savepoints WHERE a BETWEEN 18 AND 22;

DROP TABLE savepoints;

SAVEPOINT one;

ROLLBACK TO SAVEPOINT one;

RELEASE SAVEPOINT one;

BEGIN;

SAVEPOINT one;

SELECT 0/0;

SAVEPOINT two;

RELEASE SAVEPOINT one;

ROLLBACK TO SAVEPOINT one;

SELECT 1;

COMMIT;

SELECT 1;

BEGIN;

DECLARE c CURSOR FOR SELECT unique2 FROM tenk1 ORDER BY unique2;

SAVEPOINT one;

FETCH 10 FROM c;

ROLLBACK TO SAVEPOINT one;

FETCH 10 FROM c;

RELEASE SAVEPOINT one;

FETCH 10 FROM c;

CLOSE c;

DECLARE c CURSOR FOR SELECT unique2/0 FROM tenk1 ORDER BY unique2;

SAVEPOINT two;

FETCH 10 FROM c;

ROLLBACK TO SAVEPOINT two;

FETCH 10 FROM c;

ROLLBACK TO SAVEPOINT two;

RELEASE SAVEPOINT two;

FETCH 10 FROM c;

COMMIT;

select * from xacttest;

create or replace function max_xacttest() returns smallint language sql as
'select max(a) from xacttest' stable;

begin;

update xacttest set a = max_xacttest() + 10 where a > 0;

select * from xacttest;

rollback;

create or replace function max_xacttest() returns smallint language sql as
'select max(a) from xacttest' volatile;

begin;

update xacttest set a = max_xacttest() + 10 where a > 0;

select * from xacttest;

rollback;

create or replace function max_xacttest() returns smallint language plpgsql as
'begin return max(a) from xacttest; end' stable;

begin;

update xacttest set a = max_xacttest() + 10 where a > 0;

select * from xacttest;

rollback;

create or replace function max_xacttest() returns smallint language plpgsql as
'begin return max(a) from xacttest; end' volatile;

begin;

update xacttest set a = max_xacttest() + 10 where a > 0;

select * from xacttest;

rollback;

BEGIN;

savepoint x;

CREATE TABLE koju (a INT UNIQUE);

INSERT INTO koju VALUES (1);

INSERT INTO koju VALUES (1);

rollback to x;

CREATE TABLE koju (a INT UNIQUE);

INSERT INTO koju VALUES (1);

INSERT INTO koju VALUES (1);

ROLLBACK;

DROP TABLE trans_foo;

DROP TABLE trans_baz;

DROP TABLE trans_barbaz;

create function inverse(int) returns float8 as
$$
begin
  analyze revalidate_bug;
  return 1::float8/$1;
exception
  when division_by_zero then return 0;
end$$ language plpgsql volatile;

create table revalidate_bug (c float8 unique);

insert into revalidate_bug values (1);

insert into revalidate_bug values (inverse(0));

drop table revalidate_bug;

drop function inverse(int);

begin;

savepoint x;

create table trans_abc (a int);

insert into trans_abc values (5);

insert into trans_abc values (10);

declare foo cursor for select * from trans_abc;

fetch from foo;

rollback to x;

fetch from foo;

commit;

begin;

create table trans_abc (a int);

insert into trans_abc values (5);

insert into trans_abc values (10);

insert into trans_abc values (15);

declare foo cursor for select * from trans_abc;

fetch from foo;

savepoint x;

fetch from foo;

rollback to x;

fetch from foo;

abort;

CREATE FUNCTION invert(x float8) RETURNS float8 LANGUAGE plpgsql AS
$$ begin return 1/x; end $$;

CREATE FUNCTION create_temp_tab() RETURNS text
LANGUAGE plpgsql AS $$
BEGIN
  CREATE TEMP TABLE new_table (f1 float8);
  -- case of interest is that we fail while holding an open
  -- relcache reference to new_table
  INSERT INTO new_table SELECT invert(0.0);
  RETURN 'foo';
END $$;

BEGIN;

DECLARE ok CURSOR FOR SELECT * FROM int8_tbl;

DECLARE ctt CURSOR FOR SELECT create_temp_tab();

FETCH ok;

SAVEPOINT s1;

FETCH ok;

FETCH ctt;

ROLLBACK TO s1;

FETCH ok;

FETCH ctt;

COMMIT;

DROP FUNCTION create_temp_tab();

DROP FUNCTION invert(x float8);

CREATE TABLE trans_abc (a int);

SET default_transaction_read_only = on;

START TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ WRITE, DEFERRABLE;

SHOW transaction_isolation;

SHOW transaction_read_only;

SHOW transaction_deferrable;

INSERT INTO trans_abc VALUES (1);

INSERT INTO trans_abc VALUES (2);

COMMIT AND CHAIN;

SHOW transaction_isolation;

SHOW transaction_read_only;

SHOW transaction_deferrable;

INSERT INTO trans_abc VALUES ('error');

INSERT INTO trans_abc VALUES (3);

COMMIT AND CHAIN;

SHOW transaction_isolation;

SHOW transaction_read_only;

SHOW transaction_deferrable;

INSERT INTO trans_abc VALUES (4);

COMMIT;

START TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ WRITE, DEFERRABLE;

SHOW transaction_isolation;

SHOW transaction_read_only;

SHOW transaction_deferrable;

SAVEPOINT x;

INSERT INTO trans_abc VALUES ('error');

COMMIT AND CHAIN;

SHOW transaction_isolation;

SHOW transaction_read_only;

SHOW transaction_deferrable;

INSERT INTO trans_abc VALUES (5);

COMMIT;

START TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ WRITE, DEFERRABLE;

SHOW transaction_isolation;

SHOW transaction_read_only;

SHOW transaction_deferrable;

SAVEPOINT x;

COMMIT AND CHAIN;

SHOW transaction_isolation;

SHOW transaction_read_only;

SHOW transaction_deferrable;

COMMIT;

START TRANSACTION ISOLATION LEVEL READ COMMITTED, READ WRITE, DEFERRABLE;

SHOW transaction_isolation;

SHOW transaction_read_only;

SHOW transaction_deferrable;

SAVEPOINT x;

COMMIT AND CHAIN;

SHOW transaction_isolation;

SHOW transaction_read_only;

SHOW transaction_deferrable;

COMMIT;

START TRANSACTION ISOLATION LEVEL SERIALIZABLE, READ WRITE, NOT DEFERRABLE;

SHOW transaction_isolation;

SHOW transaction_read_only;

SHOW transaction_deferrable;

INSERT INTO trans_abc VALUES (6);

ROLLBACK AND CHAIN;

SHOW transaction_isolation;

SHOW transaction_read_only;

SHOW transaction_deferrable;

INSERT INTO trans_abc VALUES ('error');

ROLLBACK AND CHAIN;

SHOW transaction_isolation;

SHOW transaction_read_only;

SHOW transaction_deferrable;

ROLLBACK;

COMMIT AND CHAIN;

ROLLBACK AND CHAIN;

SELECT * FROM trans_abc ORDER BY 1;

RESET default_transaction_read_only;

DROP TABLE trans_abc;

create temp table i_table (f1 int);

SELECT 3;

select * from i_table;

select 1/0;

select * from i_table;

rollback;

commit;

rollback;

rollback;

rollback;

insert into i_table values(5);

commit;

insert into i_table values(6);

rollback;

select 1/0;

select 2;

select * from i_table;

rollback;

VACUUM;

VACUUM;

SAVEPOINT sp;

SAVEPOINT sp;

SELECT 2;

SELECT 3;

COMMIT;

COMMIT AND CHAIN;

SHOW transaction_read_only;

ROLLBACK AND CHAIN;

SHOW transaction_read_only;

CREATE TABLE trans_abc (a int);

COMMIT AND CHAIN;

ROLLBACK AND CHAIN;

COMMIT;

ROLLBACK;

COMMIT AND CHAIN;

SHOW transaction_isolation;

COMMIT;

ROLLBACK AND CHAIN;

SHOW transaction_isolation;

ROLLBACK;

SET default_transaction_isolation = 'read committed';

COMMIT AND CHAIN;

SHOW transaction_isolation;

ROLLBACK AND CHAIN;

SHOW transaction_isolation;

RESET default_transaction_isolation;

SELECT * FROM trans_abc ORDER BY 1;

DROP TABLE trans_abc;

BEGIN TRANSACTION ISOLATION LEVEL REPEATABLE READ;

SET TRANSACTION SNAPSHOT 'Incorrect Identifier';

ROLLBACK;

BEGIN TRANSACTION ISOLATION LEVEL REPEATABLE READ;

SET TRANSACTION SNAPSHOT 'FFF-FFF-F';

ROLLBACK;

begin;

select 1/0;

rollback to X;
