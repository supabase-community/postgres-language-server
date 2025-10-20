CREATE TABLE a (aa TEXT);

CREATE TABLE b (bb TEXT) INHERITS (a);

CREATE TABLE c (cc TEXT) INHERITS (a);

CREATE TABLE d (dd TEXT) INHERITS (b,c,a);

INSERT INTO a(aa) VALUES('aaa');

INSERT INTO a(aa) VALUES('aaaa');

INSERT INTO a(aa) VALUES('aaaaa');

INSERT INTO a(aa) VALUES('aaaaaa');

INSERT INTO a(aa) VALUES('aaaaaaa');

INSERT INTO a(aa) VALUES('aaaaaaaa');

INSERT INTO b(aa) VALUES('bbb');

INSERT INTO b(aa) VALUES('bbbb');

INSERT INTO b(aa) VALUES('bbbbb');

INSERT INTO b(aa) VALUES('bbbbbb');

INSERT INTO b(aa) VALUES('bbbbbbb');

INSERT INTO b(aa) VALUES('bbbbbbbb');

INSERT INTO c(aa) VALUES('ccc');

INSERT INTO c(aa) VALUES('cccc');

INSERT INTO c(aa) VALUES('ccccc');

INSERT INTO c(aa) VALUES('cccccc');

INSERT INTO c(aa) VALUES('ccccccc');

INSERT INTO c(aa) VALUES('cccccccc');

INSERT INTO d(aa) VALUES('ddd');

INSERT INTO d(aa) VALUES('dddd');

INSERT INTO d(aa) VALUES('ddddd');

INSERT INTO d(aa) VALUES('dddddd');

INSERT INTO d(aa) VALUES('ddddddd');

INSERT INTO d(aa) VALUES('dddddddd');

SELECT relname, a.* FROM a, pg_class where a.tableoid = pg_class.oid;

SELECT relname, b.* FROM b, pg_class where b.tableoid = pg_class.oid;

SELECT relname, c.* FROM c, pg_class where c.tableoid = pg_class.oid;

SELECT relname, d.* FROM d, pg_class where d.tableoid = pg_class.oid;

SELECT relname, a.* FROM ONLY a, pg_class where a.tableoid = pg_class.oid;

SELECT relname, b.* FROM ONLY b, pg_class where b.tableoid = pg_class.oid;

SELECT relname, c.* FROM ONLY c, pg_class where c.tableoid = pg_class.oid;

SELECT relname, d.* FROM ONLY d, pg_class where d.tableoid = pg_class.oid;

UPDATE a SET aa='zzzz' WHERE aa='aaaa';

UPDATE ONLY a SET aa='zzzzz' WHERE aa='aaaaa';

UPDATE b SET aa='zzz' WHERE aa='aaa';

UPDATE ONLY b SET aa='zzz' WHERE aa='aaa';

UPDATE a SET aa='zzzzzz' WHERE aa LIKE 'aaa%';

SELECT relname, a.* FROM a, pg_class where a.tableoid = pg_class.oid;

SELECT relname, b.* FROM b, pg_class where b.tableoid = pg_class.oid;

SELECT relname, c.* FROM c, pg_class where c.tableoid = pg_class.oid;

SELECT relname, d.* FROM d, pg_class where d.tableoid = pg_class.oid;

SELECT relname, a.* FROM ONLY a, pg_class where a.tableoid = pg_class.oid;

SELECT relname, b.* FROM ONLY b, pg_class where b.tableoid = pg_class.oid;

SELECT relname, c.* FROM ONLY c, pg_class where c.tableoid = pg_class.oid;

SELECT relname, d.* FROM ONLY d, pg_class where d.tableoid = pg_class.oid;

UPDATE b SET aa='new';

SELECT relname, a.* FROM a, pg_class where a.tableoid = pg_class.oid;

SELECT relname, b.* FROM b, pg_class where b.tableoid = pg_class.oid;

SELECT relname, c.* FROM c, pg_class where c.tableoid = pg_class.oid;

SELECT relname, d.* FROM d, pg_class where d.tableoid = pg_class.oid;

SELECT relname, a.* FROM ONLY a, pg_class where a.tableoid = pg_class.oid;

SELECT relname, b.* FROM ONLY b, pg_class where b.tableoid = pg_class.oid;

SELECT relname, c.* FROM ONLY c, pg_class where c.tableoid = pg_class.oid;

SELECT relname, d.* FROM ONLY d, pg_class where d.tableoid = pg_class.oid;

UPDATE a SET aa='new';

DELETE FROM ONLY c WHERE aa='new';

SELECT relname, a.* FROM a, pg_class where a.tableoid = pg_class.oid;

SELECT relname, b.* FROM b, pg_class where b.tableoid = pg_class.oid;

SELECT relname, c.* FROM c, pg_class where c.tableoid = pg_class.oid;

SELECT relname, d.* FROM d, pg_class where d.tableoid = pg_class.oid;

SELECT relname, a.* FROM ONLY a, pg_class where a.tableoid = pg_class.oid;

SELECT relname, b.* FROM ONLY b, pg_class where b.tableoid = pg_class.oid;

SELECT relname, c.* FROM ONLY c, pg_class where c.tableoid = pg_class.oid;

SELECT relname, d.* FROM ONLY d, pg_class where d.tableoid = pg_class.oid;

DELETE FROM a;

SELECT relname, a.* FROM a, pg_class where a.tableoid = pg_class.oid;

SELECT relname, b.* FROM b, pg_class where b.tableoid = pg_class.oid;

SELECT relname, c.* FROM c, pg_class where c.tableoid = pg_class.oid;

SELECT relname, d.* FROM d, pg_class where d.tableoid = pg_class.oid;

SELECT relname, a.* FROM ONLY a, pg_class where a.tableoid = pg_class.oid;

SELECT relname, b.* FROM ONLY b, pg_class where b.tableoid = pg_class.oid;

SELECT relname, c.* FROM ONLY c, pg_class where c.tableoid = pg_class.oid;

SELECT relname, d.* FROM ONLY d, pg_class where d.tableoid = pg_class.oid;

CREATE TEMP TABLE z (b TEXT, PRIMARY KEY(aa, b)) inherits (a);

INSERT INTO z VALUES (NULL, 'text');

CREATE TEMP TABLE z2 (b TEXT, UNIQUE(aa, b)) inherits (a);

INSERT INTO z2 VALUES (NULL, 'text');

create table some_tab (f1 int, f2 int, f3 int, check (f1 < 10) no inherit);

create table some_tab_child () inherits(some_tab);

insert into some_tab_child select i, i+1, 0 from generate_series(1,1000) i;

create index on some_tab_child(f1, f2);

create function some_tab_stmt_trig_func() returns trigger as
$$begin raise notice 'updating some_tab'; return NULL; end;$$
language plpgsql;

create trigger some_tab_stmt_trig
  before update on some_tab execute function some_tab_stmt_trig_func();

update some_tab set f3 = 11 where f1 = 12 and f2 = 13;

update some_tab set f3 = 11 where f1 = 12 and f2 = 13;

drop table some_tab cascade;

drop function some_tab_stmt_trig_func();

create table some_tab (a int, b int);

create table some_tab_child () inherits (some_tab);

insert into some_tab_child values(1,2);

update some_tab set a = a + 1 where false;

update some_tab set a = a + 1 where false;

update some_tab set a = a + 1 where false returning b, a;

update some_tab set a = a + 1 where false returning b, a;

table some_tab;

drop table some_tab cascade;

create temp table foo(f1 int, f2 int);

create temp table foo2(f3 int) inherits (foo);

create temp table bar(f1 int, f2 int);

create temp table bar2(f3 int) inherits (bar);

insert into foo values(1,1);

insert into foo values(3,3);

insert into foo2 values(2,2,2);

insert into foo2 values(3,3,3);

insert into bar values(1,1);

insert into bar values(2,2);

insert into bar values(3,3);

insert into bar values(4,4);

insert into bar2 values(1,1,1);

insert into bar2 values(2,2,2);

insert into bar2 values(3,3,3);

insert into bar2 values(4,4,4);

update bar set f2 = f2 + 100 where f1 in (select f1 from foo);

select tableoid::regclass::text as relname, bar.* from bar order by 1,2;

update bar set f2 = f2 + 100
from
  ( select f1 from foo union all select f1+3 from foo ) ss
where bar.f1 = ss.f1;

select tableoid::regclass::text as relname, bar.* from bar order by 1,2;

create table some_tab (a int);

insert into some_tab values (0);

create table some_tab_child () inherits (some_tab);

insert into some_tab_child values (1);

create table parted_tab (a int, b char) partition by list (a);

create table parted_tab_part1 partition of parted_tab for values in (1);

create table parted_tab_part2 partition of parted_tab for values in (2);

create table parted_tab_part3 partition of parted_tab for values in (3);

insert into parted_tab values (1, 'a'), (2, 'a'), (3, 'a');

update parted_tab set b = 'b'
from
  (select a from some_tab union all select a+1 from some_tab) ss (a)
where parted_tab.a = ss.a;

select tableoid::regclass::text as relname, parted_tab.* from parted_tab order by 1,2;

truncate parted_tab;

insert into parted_tab values (1, 'a'), (2, 'a'), (3, 'a');

update parted_tab set b = 'b'
from
  (select 0 from parted_tab union all select 1 from parted_tab) ss (a)
where parted_tab.a = ss.a;

select tableoid::regclass::text as relname, parted_tab.* from parted_tab order by 1,2;

update parted_tab set a = 2 where false;

drop table parted_tab;

create table mlparted_tab (a int, b char, c text) partition by list (a);

create table mlparted_tab_part1 partition of mlparted_tab for values in (1);

create table mlparted_tab_part2 partition of mlparted_tab for values in (2) partition by list (b);

create table mlparted_tab_part3 partition of mlparted_tab for values in (3);

create table mlparted_tab_part2a partition of mlparted_tab_part2 for values in ('a');

create table mlparted_tab_part2b partition of mlparted_tab_part2 for values in ('b');

insert into mlparted_tab values (1, 'a'), (2, 'a'), (2, 'b'), (3, 'a');

update mlparted_tab mlp set c = 'xxx'
from
  (select a from some_tab union all select a+1 from some_tab) ss (a)
where (mlp.a = ss.a and mlp.b = 'b') or mlp.a = 3;

select tableoid::regclass::text as relname, mlparted_tab.* from mlparted_tab order by 1,2;

drop table mlparted_tab;

drop table some_tab cascade;

CREATE TABLE firstparent (tomorrow date default now()::date + 1);

CREATE TABLE secondparent (tomorrow date default  now() :: date  +  1);

CREATE TABLE jointchild () INHERITS (firstparent, secondparent);

CREATE TABLE thirdparent (tomorrow date default now()::date - 1);

CREATE TABLE otherchild () INHERITS (firstparent, thirdparent);

CREATE TABLE otherchild (tomorrow date default now())
  INHERITS (firstparent, thirdparent);

DROP TABLE firstparent, secondparent, jointchild, thirdparent, otherchild;

insert into d values('test','one','two','three');

alter table a alter column aa type integer using bit_length(aa);

select * from d;

create temp table parent1(f1 int, f2 int);

create temp table parent2(f1 int, f3 bigint);

create temp table childtab(f4 int) inherits(parent1, parent2);

alter table parent1 alter column f1 type bigint;

alter table parent1 alter column f2 type bigint;

create table p1(ff1 int);

alter table p1 add constraint p1chk check (ff1 > 0) no inherit;

alter table p1 add constraint p2chk check (ff1 > 10);

select pc.relname, pgc.conname, pgc.contype, pgc.conislocal, pgc.coninhcount, pgc.connoinherit from pg_class as pc inner join pg_constraint as pgc on (pgc.conrelid = pc.oid) where pc.relname = 'p1' order by 1,2;

create table c1 () inherits (p1);

create table c2 (constraint p2chk check (ff1 > 10) no inherit) inherits (p1);

drop table p1 cascade;

create table base (i integer);

create table derived () inherits (base);

create table more_derived (like derived, b int) inherits (derived);

insert into derived (i) values (0);

select derived::base from derived;

select NULL::derived::base;

select row(i, b)::more_derived::derived::base from more_derived;

select (1, 2)::more_derived::derived::base;

drop table more_derived;

drop table derived;

drop table base;

create table p1(ff1 int);

create table p2(f1 text);

create function p2text(p2) returns text as 'select $1.f1' language sql;

create table c1(f3 int) inherits(p1,p2);

insert into c1 values(123456789, 'hi', 42);

select p2text(c1.*) from c1;

drop function p2text(p2);

drop table c1;

drop table p2;

drop table p1;

CREATE TABLE ac (aa TEXT);

alter table ac add constraint ac_check check (aa is not null);

CREATE TABLE bc (bb TEXT) INHERITS (ac);

select pc.relname, pgc.conname, pgc.contype, pgc.conislocal, pgc.coninhcount, pg_get_expr(pgc.conbin, pc.oid) as consrc from pg_class as pc inner join pg_constraint as pgc on (pgc.conrelid = pc.oid) where pc.relname in ('ac', 'bc') order by 1,2;

insert into ac (aa) values (NULL);

insert into bc (aa) values (NULL);

alter table bc drop constraint ac_check;

alter table ac drop constraint ac_check;

select pc.relname, pgc.conname, pgc.contype, pgc.conislocal, pgc.coninhcount, pg_get_expr(pgc.conbin, pc.oid) as consrc from pg_class as pc inner join pg_constraint as pgc on (pgc.conrelid = pc.oid) where pc.relname in ('ac', 'bc') order by 1,2;

alter table ac add check (aa is not null);

select pc.relname, pgc.conname, pgc.contype, pgc.conislocal, pgc.coninhcount, pg_get_expr(pgc.conbin, pc.oid) as consrc from pg_class as pc inner join pg_constraint as pgc on (pgc.conrelid = pc.oid) where pc.relname in ('ac', 'bc') order by 1,2;

insert into ac (aa) values (NULL);

insert into bc (aa) values (NULL);

alter table bc drop constraint ac_aa_check;

alter table ac drop constraint ac_aa_check;

select pc.relname, pgc.conname, pgc.contype, pgc.conislocal, pgc.coninhcount, pg_get_expr(pgc.conbin, pc.oid) as consrc from pg_class as pc inner join pg_constraint as pgc on (pgc.conrelid = pc.oid) where pc.relname in ('ac', 'bc') order by 1,2;

alter table ac add constraint ac_check check (aa is not null);

alter table bc no inherit ac;

select pc.relname, pgc.conname, pgc.contype, pgc.conislocal, pgc.coninhcount, pg_get_expr(pgc.conbin, pc.oid) as consrc from pg_class as pc inner join pg_constraint as pgc on (pgc.conrelid = pc.oid) where pc.relname in ('ac', 'bc') order by 1,2;

alter table bc drop constraint ac_check;

select pc.relname, pgc.conname, pgc.contype, pgc.conislocal, pgc.coninhcount, pg_get_expr(pgc.conbin, pc.oid) as consrc from pg_class as pc inner join pg_constraint as pgc on (pgc.conrelid = pc.oid) where pc.relname in ('ac', 'bc') order by 1,2;

alter table ac drop constraint ac_check;

select pc.relname, pgc.conname, pgc.contype, pgc.conislocal, pgc.coninhcount, pg_get_expr(pgc.conbin, pc.oid) as consrc from pg_class as pc inner join pg_constraint as pgc on (pgc.conrelid = pc.oid) where pc.relname in ('ac', 'bc') order by 1,2;

drop table bc;

drop table ac;

create table ac (a int constraint check_a check (a <> 0));

create table bc (a int constraint check_a check (a <> 0), b int constraint check_b check (b <> 0)) inherits (ac);

select pc.relname, pgc.conname, pgc.contype, pgc.conislocal, pgc.coninhcount, pg_get_expr(pgc.conbin, pc.oid) as consrc from pg_class as pc inner join pg_constraint as pgc on (pgc.conrelid = pc.oid) where pc.relname in ('ac', 'bc') order by 1,2;

drop table bc;

drop table ac;

create table ac (a int constraint check_a check (a <> 0));

create table bc (b int constraint check_b check (b <> 0));

create table cc (c int constraint check_c check (c <> 0)) inherits (ac, bc);

select pc.relname, pgc.conname, pgc.contype, pgc.conislocal, pgc.coninhcount, pg_get_expr(pgc.conbin, pc.oid) as consrc from pg_class as pc inner join pg_constraint as pgc on (pgc.conrelid = pc.oid) where pc.relname in ('ac', 'bc', 'cc') order by 1,2;

alter table cc no inherit bc;

select pc.relname, pgc.conname, pgc.contype, pgc.conislocal, pgc.coninhcount, pg_get_expr(pgc.conbin, pc.oid) as consrc from pg_class as pc inner join pg_constraint as pgc on (pgc.conrelid = pc.oid) where pc.relname in ('ac', 'bc', 'cc') order by 1,2;

drop table cc;

drop table bc;

drop table ac;

create table p1(f1 int);

create table p2(f2 int);

create table c1(f3 int) inherits(p1,p2);

insert into c1 values(1,-1,2);

alter table p2 add constraint cc check (f2>0);

alter table p2 add check (f2>0);

delete from c1;

insert into c1 values(1,1,2);

alter table p2 add check (f2>0);

insert into c1 values(1,-1,2);

create table c2(f3 int) inherits(p1,p2);

create table c3 (f4 int) inherits(c1,c2);

drop table p1 cascade;

drop table p2 cascade;

create table pp1 (f1 int);

create table cc1 (f2 text, f3 int) inherits (pp1);

alter table pp1 add column a1 int check (a1 > 0);

create table cc2(f4 float) inherits(pp1,cc1);

alter table pp1 add column a2 int check (a2 > 0);

drop table pp1 cascade;

CREATE TABLE inht1 (a int, b int);

CREATE TABLE inhs1 (b int, c int);

CREATE TABLE inhts (d int) INHERITS (inht1, inhs1);

ALTER TABLE inht1 RENAME a TO aa;

ALTER TABLE inht1 RENAME b TO bb;

ALTER TABLE inhts RENAME aa TO aaa;

ALTER TABLE inhts RENAME d TO dd;

DROP TABLE inhts;

CREATE TABLE inhta ();

CREATE TABLE inhtb () INHERITS (inhta);

CREATE TABLE inhtc () INHERITS (inhtb);

CREATE TABLE inhtd () INHERITS (inhta, inhtb, inhtc);

ALTER TABLE inhta ADD COLUMN i int, ADD COLUMN j bigint DEFAULT 1;

DROP TABLE inhta, inhtb, inhtc, inhtd;

CREATE TABLE inht2 (x int) INHERITS (inht1);

CREATE TABLE inht3 (y int) INHERITS (inht1);

CREATE TABLE inht4 (z int) INHERITS (inht2, inht3);

ALTER TABLE inht1 RENAME aa TO aaa;

CREATE TABLE inhts (d int) INHERITS (inht2, inhs1);

ALTER TABLE inht1 RENAME aaa TO aaaa;

ALTER TABLE inht1 RENAME b TO bb;

WITH RECURSIVE r AS (
  SELECT 'inht1'::regclass AS inhrelid
UNION ALL
  SELECT c.inhrelid FROM pg_inherits c, r WHERE r.inhrelid = c.inhparent
)
SELECT a.attrelid::regclass, a.attname, a.attinhcount, e.expected
  FROM (SELECT inhrelid, count(*) AS expected FROM pg_inherits
        WHERE inhparent IN (SELECT inhrelid FROM r) GROUP BY inhrelid) e
  JOIN pg_attribute a ON e.inhrelid = a.attrelid WHERE NOT attislocal
  ORDER BY a.attrelid::regclass::name, a.attnum;

DROP TABLE inht1, inhs1 CASCADE;

CREATE TABLE test_constraints (id int, val1 varchar, val2 int, UNIQUE(val1, val2));

CREATE TABLE test_constraints_inh () INHERITS (test_constraints);

ALTER TABLE ONLY test_constraints DROP CONSTRAINT test_constraints_val1_val2_key;

DROP TABLE test_constraints_inh;

DROP TABLE test_constraints;

CREATE TABLE test_ex_constraints (
    c circle,
    EXCLUDE USING gist (c WITH &&)
);

CREATE TABLE test_ex_constraints_inh () INHERITS (test_ex_constraints);

ALTER TABLE test_ex_constraints DROP CONSTRAINT test_ex_constraints_c_excl;

DROP TABLE test_ex_constraints_inh;

DROP TABLE test_ex_constraints;

CREATE TABLE test_primary_constraints(id int PRIMARY KEY);

CREATE TABLE test_foreign_constraints(id1 int REFERENCES test_primary_constraints(id));

CREATE TABLE test_foreign_constraints_inh () INHERITS (test_foreign_constraints);

ALTER TABLE test_foreign_constraints DROP CONSTRAINT test_foreign_constraints_id1_fkey;

DROP TABLE test_foreign_constraints_inh;

DROP TABLE test_foreign_constraints;

DROP TABLE test_primary_constraints;

create table inh_fk_1 (a int primary key);

insert into inh_fk_1 values (1), (2), (3);

create table inh_fk_2 (x int primary key, y int references inh_fk_1 on delete cascade);

insert into inh_fk_2 values (11, 1), (22, 2), (33, 3);

create table inh_fk_2_child () inherits (inh_fk_2);

insert into inh_fk_2_child values (111, 1), (222, 2);

delete from inh_fk_1 where a = 1;

select * from inh_fk_1 order by 1;

select * from inh_fk_2 order by 1, 2;

drop table inh_fk_1, inh_fk_2, inh_fk_2_child;

create table p1(f1 int);

create table p1_c1() inherits(p1);

alter table p1 add constraint inh_check_constraint1 check (f1 > 0);

alter table p1_c1 add constraint inh_check_constraint1 check (f1 > 0);

alter table p1_c1 add constraint inh_check_constraint2 check (f1 < 10);

alter table p1 add constraint inh_check_constraint2 check (f1 < 10);

create table p1_c2(f1 int constraint inh_check_constraint4 check (f1 < 10)) inherits(p1);

create table p1_c3() inherits(p1, p1_c1);

select conrelid::regclass::text as relname, conname, conislocal, coninhcount, conenforced, convalidated
from pg_constraint where conname like 'inh\_check\_constraint%'
order by 1, 2;

drop table p1 cascade;

alter table p1_c1 inherit p1;

drop table p1 cascade;

alter table p1_c1 inherit p1;

drop table p1, p1_c1;

create table p1(f1 int constraint f1_pos CHECK (f1 > 0));

create table p1_c1 (f1 int constraint f1_pos CHECK (f1 > 0)) inherits (p1);

alter table p1_c1 drop constraint f1_pos;

alter table p1 drop constraint f1_pos;

drop table p1 cascade;

create table p1(f1 int constraint f1_pos CHECK (f1 > 0));

create table p2(f1 int constraint f1_pos CHECK (f1 > 0));

create table p1p2_c1 (f1 int) inherits (p1, p2);

create table p1p2_c2 (f1 int constraint f1_pos CHECK (f1 > 0)) inherits (p1, p2);

alter table p2 drop constraint f1_pos;

alter table p1 drop constraint f1_pos;

drop table p1, p2 cascade;

create table p1(f1 int constraint f1_pos CHECK (f1 > 0));

create table p1_c1() inherits (p1);

create table p1_c2() inherits (p1);

create table p1_c1c2() inherits (p1_c1, p1_c2);

alter table p1 drop constraint f1_pos;

drop table p1 cascade;

create table p1(f1 int constraint f1_pos CHECK (f1 > 0));

create table p1_c1() inherits (p1);

create table p1_c2(constraint f1_pos CHECK (f1 > 0)) inherits (p1);

create table p1_c1c2() inherits (p1_c1, p1_c2, p1);

alter table p1_c2 drop constraint f1_pos;

alter table p1 drop constraint f1_pos;

alter table p1_c1c2 drop constraint f1_pos;

alter table p1_c2 drop constraint f1_pos;

drop table p1 cascade;

create table invalid_check_con(f1 int);

create table invalid_check_con_child() inherits(invalid_check_con);

alter table invalid_check_con_child add constraint inh_check_constraint check(f1 > 0) not valid;

alter table invalid_check_con add constraint inh_check_constraint check(f1 > 0);

alter table invalid_check_con_child drop constraint inh_check_constraint;

insert into invalid_check_con values(0);

alter table invalid_check_con_child add constraint inh_check_constraint check(f1 > 0);

alter table invalid_check_con add constraint inh_check_constraint check(f1 > 0) not valid;

insert into invalid_check_con values(0);

insert into invalid_check_con_child values(0);

select conrelid::regclass::text as relname, conname,
       convalidated, conislocal, coninhcount, connoinherit
from pg_constraint where conname like 'inh\_check\_constraint%'
order by 1, 2;

create temp table patest0 (id, x) as
  select x, x from generate_series(0,1000) x;

create temp table patest1() inherits (patest0);

insert into patest1
  select x, x from generate_series(0,1000) x;

create temp table patest2() inherits (patest0);

insert into patest2
  select x, x from generate_series(0,1000) x;

create index patest0i on patest0(id);

create index patest1i on patest1(id);

create index patest2i on patest2(id);

analyze patest0;

analyze patest1;

analyze patest2;

select * from patest0 join (select f1 from int4_tbl limit 1) ss on id = f1;

select * from patest0 join (select f1 from int4_tbl limit 1) ss on id = f1;

drop index patest2i;

select * from patest0 join (select f1 from int4_tbl limit 1) ss on id = f1;

select * from patest0 join (select f1 from int4_tbl limit 1) ss on id = f1;

drop table patest0 cascade;

create table matest0 (id serial primary key, name text);

create table matest1 (id integer primary key) inherits (matest0);

create table matest2 (id integer primary key) inherits (matest0);

create table matest3 (id integer primary key) inherits (matest0);

create index matest0i on matest0 ((1-id));

create index matest1i on matest1 ((1-id));

create index matest3i on matest3 ((1-id));

insert into matest1 (name) values ('Test 1');

insert into matest1 (name) values ('Test 2');

insert into matest2 (name) values ('Test 3');

insert into matest2 (name) values ('Test 4');

insert into matest3 (name) values ('Test 5');

insert into matest3 (name) values ('Test 6');

set enable_indexscan = off;

select * from matest0 order by 1-id;

select * from matest0 order by 1-id;

select min(1-id) from matest0;

select min(1-id) from matest0;

reset enable_indexscan;

set enable_seqscan = off;

set enable_parallel_append = off;

select * from matest0 order by 1-id;

select * from matest0 order by 1-id;

select min(1-id) from matest0;

select min(1-id) from matest0;

reset enable_seqscan;

reset enable_parallel_append;

select 1 - id as c from
(select id from matest3 t1 union all select id * 2 from matest3 t2) ss
order by c;

select 1 - id as c from
(select id from matest3 t1 union all select id * 2 from matest3 t2) ss
order by c;

drop table matest0 cascade;

create table matest0 (a int, b int, c int, d int);

create table matest1 () inherits(matest0);

create index matest0i on matest0 (b, c);

create index matest1i on matest1 (b, c);

set enable_nestloop = off;

select t1.* from matest0 t1, matest0 t2
where t1.b = t2.b and t2.c = t2.d
order by t1.b limit 10;

reset enable_nestloop;

drop table matest0 cascade;

create table matest0(a int primary key);

create table matest1() inherits (matest0);

insert into matest0 select generate_series(1, 400);

insert into matest1 select generate_series(1, 200);

analyze matest0;

analyze matest1;

select * from matest0 where a < 100 order by a;

drop table matest0 cascade;

set enable_seqscan = off;

set enable_indexscan = on;

set enable_bitmapscan = off;

SELECT thousand, tenthous FROM tenk1
UNION ALL
SELECT thousand, thousand FROM tenk1
ORDER BY thousand, tenthous;

SELECT thousand, tenthous, thousand+tenthous AS x FROM tenk1
UNION ALL
SELECT 42, 42, hundred FROM tenk1
ORDER BY thousand, tenthous;

SELECT thousand, tenthous FROM tenk1
UNION ALL
SELECT thousand, random()::integer FROM tenk1
ORDER BY thousand, tenthous;

SELECT min(x) FROM
  (SELECT unique1 AS x FROM tenk1 a
   UNION ALL
   SELECT unique2 AS x FROM tenk1 b) s;

SELECT min(y) FROM
  (SELECT unique1 AS x, unique1 AS y FROM tenk1 a
   UNION ALL
   SELECT unique2 AS x, unique2 AS y FROM tenk1 b) s;

SELECT x, y FROM
  (SELECT thousand AS x, tenthous AS y FROM tenk1 a
   UNION ALL
   SELECT unique2 AS x, unique2 AS y FROM tenk1 b) s
ORDER BY x, y;

SELECT
    ARRAY(SELECT f.i FROM (
        (SELECT d + g.i FROM generate_series(4, 30, 3) d ORDER BY 1)
        UNION ALL
        (SELECT d + g.i FROM generate_series(0, 30, 5) d ORDER BY 1)
    ) f(i)
    ORDER BY f.i LIMIT 10)
FROM generate_series(1, 3) g(i);

SELECT
    ARRAY(SELECT f.i FROM (
        (SELECT d + g.i FROM generate_series(4, 30, 3) d ORDER BY 1)
        UNION ALL
        (SELECT d + g.i FROM generate_series(0, 30, 5) d ORDER BY 1)
    ) f(i)
    ORDER BY f.i LIMIT 10)
FROM generate_series(1, 3) g(i);

reset enable_seqscan;

reset enable_indexscan;

reset enable_bitmapscan;

create table inhpar(f1 int, f2 name);

create table inhcld(f2 name, f1 int);

alter table inhcld inherit inhpar;

insert into inhpar select x, x::text from generate_series(1,5) x;

insert into inhcld select x::text, x from generate_series(6,10) x;

update inhpar i set (f1, f2) = (select i.f1, i.f2 || '-' from int4_tbl limit 1);

update inhpar i set (f1, f2) = (select i.f1, i.f2 || '-' from int4_tbl limit 1);

select * from inhpar;

drop table inhpar cascade;

create table inhpar(f1 int primary key, f2 name) partition by range (f1);

create table inhcld1(f2 name, f1 int primary key);

create table inhcld2(f1 int primary key, f2 name);

alter table inhpar attach partition inhcld1 for values from (1) to (5);

alter table inhpar attach partition inhcld2 for values from (5) to (100);

insert into inhpar select x, x::text from generate_series(1,10) x;

update inhpar i set (f1, f2) = (select i.f1, i.f2 || '-' from int4_tbl limit 1);

update inhpar i set (f1, f2) = (select i.f1, i.f2 || '-' from int4_tbl limit 1);

select * from inhpar;

insert into inhpar as i values (3), (7) on conflict (f1)
  do update set (f1, f2) = (select i.f1, i.f2 || '+');

select * from inhpar order by f1;

drop table inhpar cascade;

create table cnullparent (f1 int);

create table cnullchild (check (f1 = 1 or f1 = null)) inherits(cnullparent);

insert into cnullchild values(1);

insert into cnullchild values(2);

insert into cnullchild values(null);

select * from cnullparent;

select * from cnullparent where f1 = 2;

drop table cnullparent cascade;

create table pp1 (f1 int);

create table cc1 (f2 text, f3 int) inherits (pp1);

create table cc2 (f4 float) inherits (pp1,cc1);

create table cc3 () inherits (pp1,cc1,cc2);

alter table pp1 alter f1 set not null;

alter table cc3 no inherit pp1;

alter table cc3 no inherit cc1;

alter table cc3 no inherit cc2;

drop table cc3;

alter table cc1 add column a2 int constraint nn not null;

alter table pp1 alter column f1 set not null;

alter table cc2 alter column a2 drop not null;

alter table cc1 alter column a2 drop not null;

alter table cc2 alter column f1 drop not null;

alter table cc1 alter column f1 drop not null;

alter table pp1 alter column f1 drop not null;

alter table pp1 add primary key (f1);

alter table inh_child inherit inh_parent;

alter table inh_child no inherit inh_parent;

drop table inh_parent, inh_child;

create table inh_pp1 (f1 int);

create table inh_cc1 (f2 text, f3 int) inherits (inh_pp1);

create table inh_cc2(f4 float) inherits(inh_pp1,inh_cc1);

alter table inh_pp1 alter column f1 set not null;

alter table inh_cc2 no inherit inh_pp1;

alter table inh_cc2 no inherit inh_cc1;

drop table inh_pp1, inh_cc1, inh_cc2;

create table inh_pp1 (f1 int not null);

create table inh_cc1 (f2 text, f3 int) inherits (inh_pp1);

create table inh_cc2(f4 float) inherits(inh_pp1,inh_cc1);

alter table inh_pp1 alter column f1 drop not null;

drop table inh_pp1, inh_cc1, inh_cc2;

CREATE TABLE inh_parent ();

CREATE TABLE inh_child (i int) INHERITS (inh_parent);

CREATE TABLE inh_grandchild () INHERITS (inh_parent, inh_child);

ALTER TABLE inh_parent ADD COLUMN i int NOT NULL;

drop table inh_parent, inh_child, inh_grandchild;

create table inh_parent1(a int constraint nn not null);

create table inh_parent2(b int constraint nn not null);

create table inh_child1 () inherits (inh_parent1, inh_parent2);

alter table inh_child2 no inherit inh_parent2;

drop table inh_parent1, inh_parent2, inh_child1, inh_child2;

create table inh_parent1(a int, b int, c int, primary key (a, b));

create table inh_parent2(d int, e int, b int, primary key (d, b));

create table inh_child() inherits (inh_parent1, inh_parent2);

select conrelid::regclass, conname, contype, conkey,
 coninhcount, conislocal, connoinherit
 from pg_constraint where contype in ('n','p') and
 conrelid::regclass::text in ('inh_child', 'inh_parent1', 'inh_parent2')
 order by 1, 2;

drop table inh_parent1, inh_parent2, inh_child;

create table inh_nn_parent(a int);

create table inh_nn_child() inherits (inh_nn_parent);

create table inh_nn_child2() inherits (inh_nn_parent);

select conrelid::regclass, conname, contype, conkey,
 (select attname from pg_attribute where attrelid = conrelid and attnum = conkey[1]),
 coninhcount, conislocal, connoinherit
 from pg_constraint where contype = 'n' and
 conrelid::regclass::text like 'inh\_nn\_%'
 order by 2, 1;

drop table inh_nn_parent, inh_nn_child, inh_nn_child2;

CREATE TABLE inh_nn_child() INHERITS (inh_nn_parent);

ALTER TABLE inh_nn_parent ALTER a SET NOT NULL;

DROP TABLE inh_nn_parent cascade;

CREATE TABLE inh_nn_lvl1 (a int);

CREATE TABLE inh_nn_lvl2 () INHERITS (inh_nn_lvl1);

ALTER TABLE inh_nn_lvl1 ADD PRIMARY KEY (a);

DROP TABLE inh_nn_lvl1, inh_nn_lvl2, inh_nn_lvl3;

CREATE TABLE inh_nn1 (a int not null);

DROP TABLE IF EXISTS inh_nn1, inh_nn2, inh_nn3, inh_nn4;

create table inh_parent(f1 int);

create table inh_child1(f1 int not null);

create table inh_child2(f1 int);

alter table inh_child1 inherit inh_parent;

alter table inh_child2 inherit inh_child1;

alter table inh_child2 alter column f1 set not null;

alter table inh_child2 inherit inh_child1;

alter table inh_parent alter column f1 set not null;

select conrelid::regclass, conname, contype, coninhcount, conislocal
 from pg_constraint where contype = 'n' and
 conrelid in ('inh_parent'::regclass, 'inh_child1'::regclass, 'inh_child2'::regclass)
 order by 2, 1;

create table inh_child3 () inherits (inh_child1);

alter table inh_child1 no inherit inh_parent;

select conrelid::regclass, conname, contype, coninhcount, conislocal
 from pg_constraint where contype = 'n' and
 conrelid::regclass::text in ('inh_parent', 'inh_child1', 'inh_child2', 'inh_child3')
 order by 2, 1;

drop table inh_parent, inh_child1, inh_child2, inh_child3;

create table inh_parent (a int not null);

create table inh_child (a int);

alter table inh_child inherit inh_parent;

drop table inh_parent, inh_child;

create table inh_parent (a int not null);

alter table inh_child inherit inh_parent;

drop table inh_parent, inh_child;

create table inh_parent (a int primary key);

create table inh_child (a int primary key) inherits (inh_parent);

alter table inh_parent add constraint inh_parent_excl exclude ((1) with =);

alter table inh_parent add constraint inh_parent_uq unique (a);

alter table inh_parent add constraint inh_parent_fk foreign key (a) references inh_parent (a);

create table inh_child2 () inherits (inh_parent);

create table inh_child3 (like inh_parent);

alter table inh_child3 inherit inh_parent;

select conrelid::regclass, conname, contype, coninhcount, conislocal
 from pg_constraint
 where conrelid::regclass::text in ('inh_parent', 'inh_child', 'inh_child2', 'inh_child3')
 order by 2, 1;

drop table inh_parent, inh_child, inh_child2, inh_child3;

create table inh_parent(f1 int not null);

create table inh_child1() inherits(inh_parent);

create table inh_child2() inherits(inh_parent);

create table inh_child3() inherits(inh_child1, inh_child2);

select conrelid::regclass, conname, contype, coninhcount, conislocal
 from pg_constraint where contype = 'n' and
 conrelid in ('inh_parent'::regclass, 'inh_child1'::regclass, 'inh_child2'::regclass, 'inh_child3'::regclass)
 order by 2, conrelid::regclass::text;

drop table inh_parent cascade;

create table inh_parent_1(f1 int);

create table inh_parent_2(f2 text);

create table inh_child(f1 int not null, f2 text not null) inherits(inh_parent_1, inh_parent_2);

select conrelid::regclass, conname, contype, coninhcount, conislocal
 from pg_constraint where contype = 'n' and
 conrelid in ('inh_parent_1'::regclass, 'inh_parent_2'::regclass, 'inh_child'::regclass)
 order by 2, conrelid::regclass::text;

drop table inh_parent_1 cascade;

drop table inh_parent_2;

create table inh_p1(f1 int not null);

create table inh_p2(f1 int not null);

create table inh_p3(f2 int);

create table inh_p4(f1 int not null, f3 text not null);

create table inh_multiparent() inherits(inh_p1, inh_p2, inh_p3, inh_p4);

select conrelid::regclass, contype, conname,
  (select attname from pg_attribute where attrelid = conrelid and attnum = conkey[1]),
  coninhcount, conislocal
 from pg_constraint where contype = 'n' and
 conrelid::regclass in ('inh_p1', 'inh_p2', 'inh_p3', 'inh_p4',
	'inh_multiparent')
 order by conrelid::regclass::text, conname;

create table inh_multiparent2 (a int not null, f1 int) inherits(inh_p3, inh_multiparent);

select conrelid::regclass, contype, conname,
  (select attname from pg_attribute where attrelid = conrelid and attnum = conkey[1]),
  coninhcount, conislocal
 from pg_constraint where contype = 'n' and
 conrelid::regclass in ('inh_p3', 'inh_multiparent', 'inh_multiparent2')
 order by conrelid::regclass::text, conname;

drop table inh_p1, inh_p2, inh_p3, inh_p4 cascade;

create table inh_nn2 (f2 text, f3 int, f1 int);

alter table inh_nn2 inherit inh_nn1;

create table inh_nn3 (f4 float) inherits (inh_nn2);

create table inh_nn4 (f5 int, f4 float, f2 text, f3 int, f1 int);

alter table inh_nn4 inherit inh_nn2, inherit inh_nn1, inherit inh_nn3;

select conrelid::regclass, conname, conkey, coninhcount, conislocal, connoinherit
 from pg_constraint where contype = 'n' and
 conrelid::regclass::text in ('inh_nn1', 'inh_nn2', 'inh_nn3', 'inh_nn4')
 order by 2, 1;

select conrelid::regclass, conname, conkey, coninhcount, conislocal, connoinherit
 from pg_constraint where contype = 'n' and
 conrelid::regclass::text in ('inh_nn1', 'inh_nn2', 'inh_nn3', 'inh_nn4')
 order by 2, 1;

alter table inh_nn1 drop constraint inh_nn1_f1_not_null;

select conrelid::regclass, conname, coninhcount, conislocal, connoinherit
 from pg_constraint where contype = 'n' and
 conrelid::regclass::text in ('inh_nn1', 'inh_nn2', 'inh_nn3', 'inh_nn4')
 order by 2, 1;

drop table inh_nn1, inh_nn2, inh_nn3, inh_nn4;

create table inh_nn2 (f2 text, f3 int) inherits (inh_nn1);

insert into inh_nn2 values(NULL, 'sample', 1);

delete from inh_nn2;

create table inh_nn3 () inherits (inh_nn2);

create table inh_nn4 () inherits (inh_nn1, inh_nn2);

select conrelid::regclass, conname, coninhcount, conislocal, connoinherit
 from pg_constraint where contype = 'n' and
 conrelid::regclass::text in ('inh_nn1', 'inh_nn2', 'inh_nn3', 'inh_nn4')
 order by 2, 1;

drop table inh_nn1, inh_nn2, inh_nn3, inh_nn4;

create table inh_nn2 (f2 text, f3 int) inherits (inh_nn1);

select conrelid::regclass, conname, conkey, coninhcount, conislocal, connoinherit
 from pg_constraint where contype = 'n' and
 conrelid::regclass::text in ('inh_nn1', 'inh_nn2', 'inh_nn3')
 order by 2, 1;

select conrelid::regclass, conname, conkey, coninhcount, conislocal, connoinherit
 from pg_constraint where contype = 'n' and
 conrelid::regclass::text in ('inh_nn1', 'inh_nn2', 'inh_nn3')
 order by 2, 1;

drop table inh_nn1, inh_nn2, inh_nn3;

create table inh_nn1 (f1 int check(f1 > 5) primary key references inh_nn1, f2 int not null);

create table inh_nn2 () inherits (inh_nn1);

drop table inh_nn1, inh_nn2;

create role regress_alice;

create role regress_bob;

grant all on schema public to regress_alice, regress_bob;

grant regress_alice to regress_bob;

set session authorization regress_alice;

create table inh_parent (a int not null);

set session authorization regress_bob;

create table inh_child () inherits (inh_parent);

set session authorization regress_alice;

alter table inh_parent alter a drop not null;

set session authorization regress_bob;

alter table inh_parent alter a drop not null;

reset session authorization;

drop table inh_parent, inh_child;

revoke all on schema public from regress_alice, regress_bob;

drop role regress_alice, regress_bob;

create table inh_perm_parent (a1 int);

create temp table inh_temp_parent (a1 int);

create temp table inh_temp_child () inherits (inh_perm_parent);

create table inh_perm_child () inherits (inh_temp_parent);

create temp table inh_temp_child_2 () inherits (inh_temp_parent);

insert into inh_perm_parent values (1);

insert into inh_temp_parent values (2);

insert into inh_temp_child values (3);

insert into inh_temp_child_2 values (4);

select tableoid::regclass, a1 from inh_perm_parent;

select tableoid::regclass, a1 from inh_temp_parent;

drop table inh_perm_parent cascade;

drop table inh_temp_parent cascade;

create table list_parted (
	a	varchar
) partition by list (a);

create table part_ab_cd partition of list_parted for values in ('ab', 'cd');

create table part_ef_gh partition of list_parted for values in ('ef', 'gh');

create table part_null_xy partition of list_parted for values in (null, 'xy');

select * from list_parted;

select * from list_parted where a is null;

select * from list_parted where a is not null;

select * from list_parted where a in ('ab', 'cd', 'ef');

select * from list_parted where a = 'ab' or a in (null, 'cd');

select * from list_parted where a = 'ab';

create table range_list_parted (
	a	int,
	b	char(2)
) partition by range (a);

create table part_1_10 partition of range_list_parted for values from (1) to (10) partition by list (b);

create table part_1_10_ab partition of part_1_10 for values in ('ab');

create table part_1_10_cd partition of part_1_10 for values in ('cd');

create table part_10_20 partition of range_list_parted for values from (10) to (20) partition by list (b);

create table part_10_20_ab partition of part_10_20 for values in ('ab');

create table part_10_20_cd partition of part_10_20 for values in ('cd');

create table part_21_30 partition of range_list_parted for values from (21) to (30) partition by list (b);

create table part_21_30_ab partition of part_21_30 for values in ('ab');

create table part_21_30_cd partition of part_21_30 for values in ('cd');

create table part_40_inf partition of range_list_parted for values from (40) to (maxvalue) partition by list (b);

create table part_40_inf_ab partition of part_40_inf for values in ('ab');

create table part_40_inf_cd partition of part_40_inf for values in ('cd');

create table part_40_inf_null partition of part_40_inf for values in (null);

select * from range_list_parted;

select * from range_list_parted where a = 5;

select * from range_list_parted where b = 'ab';

select * from range_list_parted where a between 3 and 23 and b in ('ab');

select * from range_list_parted where a is null;

select * from range_list_parted where b is null;

select * from range_list_parted where a is not null and a < 67;

select * from range_list_parted where a >= 30;

drop table list_parted;

drop table range_list_parted;

create table mcrparted (a int, b int, c int) partition by range (a, abs(b), c);

create table mcrparted_def partition of mcrparted default;

create table mcrparted0 partition of mcrparted for values from (minvalue, minvalue, minvalue) to (1, 1, 1);

create table mcrparted1 partition of mcrparted for values from (1, 1, 1) to (10, 5, 10);

create table mcrparted2 partition of mcrparted for values from (10, 5, 10) to (10, 10, 10);

create table mcrparted3 partition of mcrparted for values from (11, 1, 1) to (20, 10, 10);

create table mcrparted4 partition of mcrparted for values from (20, 10, 10) to (20, 20, 20);

create table mcrparted5 partition of mcrparted for values from (20, 20, 20) to (maxvalue, maxvalue, maxvalue);

select * from mcrparted where a = 0;

select * from mcrparted where a = 10 and abs(b) < 5;

select * from mcrparted where a = 10 and abs(b) = 5;

select * from mcrparted where abs(b) = 5;

select * from mcrparted where a > -1;

select * from mcrparted where a = 20 and abs(b) = 10 and c > 10;

select * from mcrparted where a = 20 and c > 20;

create table parted_minmax (a int, b varchar(16)) partition by range (a);

create table parted_minmax1 partition of parted_minmax for values from (1) to (10);

create index parted_minmax1i on parted_minmax1 (a, b);

insert into parted_minmax values (1,'12345');

select min(a), max(a) from parted_minmax where b = '12345';

select min(a), max(a) from parted_minmax where b = '12345';

drop table parted_minmax;

create index mcrparted_a_abs_c_idx on mcrparted (a, abs(b), c);

select * from mcrparted order by a, abs(b), c;

drop table mcrparted_def;

select * from mcrparted order by a, abs(b), c;

select * from mcrparted order by a desc, abs(b) desc, c desc;

drop table mcrparted5;

create table mcrparted5 partition of mcrparted for values from (20, 20, 20) to (maxvalue, maxvalue, maxvalue) partition by list (a);

create table mcrparted5a partition of mcrparted5 for values in(20);

create table mcrparted5_def partition of mcrparted5 default;

select * from mcrparted order by a, abs(b), c;

drop table mcrparted5_def;

select a, abs(b) from mcrparted order by a, abs(b), c;

select * from mcrparted where a < 20 order by a, abs(b), c;

set enable_bitmapscan to off;

set enable_sort to off;

create table mclparted (a int) partition by list(a);

create table mclparted1 partition of mclparted for values in(1);

create table mclparted2 partition of mclparted for values in(2);

create index on mclparted (a);

select * from mclparted order by a;

create table mclparted3_5 partition of mclparted for values in(3,5);

create table mclparted4 partition of mclparted for values in(4);

select * from mclparted order by a;

select * from mclparted where a in(3,4,5) order by a;

create table mclparted_null partition of mclparted for values in(null);

create table mclparted_def partition of mclparted default;

select * from mclparted where a in(1,2,4) order by a;

select * from mclparted where a in(1,2,4) or a is null order by a;

drop table mclparted_null;

create table mclparted_0_null partition of mclparted for values in(0,null);

select * from mclparted where a in(1,2,4) or a is null order by a;

select * from mclparted where a in(0,1,2,4) order by a;

select * from mclparted where a in(1,2,4) order by a;

select * from mclparted where a in(1,2,4,100) order by a;

drop table mclparted;

reset enable_sort;

reset enable_bitmapscan;

drop index mcrparted_a_abs_c_idx;

create index on mcrparted1 (a, abs(b), c);

create index on mcrparted2 (a, abs(b), c);

create index on mcrparted3 (a, abs(b), c);

create index on mcrparted4 (a, abs(b), c);

select * from mcrparted where a < 20 order by a, abs(b), c limit 1;

set enable_bitmapscan = 0;

select * from mcrparted where a = 10 order by a, abs(b), c;

reset enable_bitmapscan;

drop table mcrparted;

create table bool_lp (b bool) partition by list(b);

create table bool_lp_true partition of bool_lp for values in(true);

create table bool_lp_false partition of bool_lp for values in(false);

create index on bool_lp (b);

select * from bool_lp order by b;

drop table bool_lp;

create table bool_rp (b bool, a int) partition by range(b,a);

create table bool_rp_false_1k partition of bool_rp for values from (false,0) to (false,1000);

create table bool_rp_true_1k partition of bool_rp for values from (true,0) to (true,1000);

create table bool_rp_false_2k partition of bool_rp for values from (false,1000) to (false,2000);

create table bool_rp_true_2k partition of bool_rp for values from (true,1000) to (true,2000);

create index on bool_rp (b,a);

select * from bool_rp where b = true order by b,a;

select * from bool_rp where b = false order by b,a;

select * from bool_rp where b = true order by a;

select * from bool_rp where b = false order by a;

drop table bool_rp;

create table range_parted (a int, b int, c int) partition by range(a, b);

create table range_parted1 partition of range_parted for values from (0,0) to (10,10);

create table range_parted2 partition of range_parted for values from (10,10) to (20,20);

create index on range_parted (a,b,c);

select * from range_parted order by a,b,c;

select * from range_parted order by a desc,b desc,c desc;

drop table range_parted;

create table permtest_parent (a int, b text, c text) partition by list (a);

create table permtest_child (b text, c text, a int) partition by list (b);

create table permtest_grandchild (c text, b text, a int);

alter table permtest_child attach partition permtest_grandchild for values in ('a');

alter table permtest_parent attach partition permtest_child for values in (1);

create index on permtest_parent (left(c, 3));

insert into permtest_parent
  select 1, 'a', left(fipshash(i::text), 5) from generate_series(0, 100) i;

analyze permtest_parent;

create role regress_no_child_access;

revoke all on permtest_grandchild from regress_no_child_access;

grant select on permtest_parent to regress_no_child_access;

set session authorization regress_no_child_access;

select * from permtest_parent p1 inner join permtest_parent p2
  on p1.a = p2.a and p1.c ~ 'a1$';

select * from permtest_parent p1 inner join permtest_parent p2
  on p1.a = p2.a and left(p1.c, 3) ~ 'a1$';

reset session authorization;

revoke all on permtest_parent from regress_no_child_access;

grant select(a,c) on permtest_parent to regress_no_child_access;

set session authorization regress_no_child_access;

select p2.a, p1.c from permtest_parent p1 inner join permtest_parent p2
  on p1.a = p2.a and p1.c ~ 'a1$';

select p2.a, p1.c from permtest_parent p1 inner join permtest_parent p2
  on p1.a = p2.a and left(p1.c, 3) ~ 'a1$';

reset session authorization;

revoke all on permtest_parent from regress_no_child_access;

drop role regress_no_child_access;

drop table permtest_parent;

CREATE TABLE errtst_parent (
    partid int not null,
    shdata int not null,
    data int NOT NULL DEFAULT 0,
    CONSTRAINT shdata_small CHECK(shdata < 3)
) PARTITION BY RANGE (partid);

CREATE TABLE errtst_child_fastdef (
    partid int not null,
    shdata int not null,
    CONSTRAINT shdata_small CHECK(shdata < 3)
);

CREATE TABLE errtst_child_plaindef (
    partid int not null,
    shdata int not null,
    data int NOT NULL DEFAULT 0,
    CONSTRAINT shdata_small CHECK(shdata < 3),
    CHECK(data < 10)
);

CREATE TABLE errtst_child_reorder (
    data int NOT NULL DEFAULT 0,
    shdata int not null,
    partid int not null,
    CONSTRAINT shdata_small CHECK(shdata < 3),
    CHECK(data < 10)
);

ALTER TABLE errtst_child_fastdef ADD COLUMN data int NOT NULL DEFAULT 0;

ALTER TABLE errtst_child_fastdef ADD CONSTRAINT errtest_child_fastdef_data_check CHECK (data < 10);

ALTER TABLE errtst_parent ATTACH PARTITION errtst_child_fastdef FOR VALUES FROM (0) TO (10);

ALTER TABLE errtst_parent ATTACH PARTITION errtst_child_plaindef FOR VALUES FROM (10) TO (20);

ALTER TABLE errtst_parent ATTACH PARTITION errtst_child_reorder FOR VALUES FROM (20) TO (30);

INSERT INTO errtst_parent(partid, shdata, data) VALUES ( '0', '1', '5');

INSERT INTO errtst_parent(partid, shdata, data) VALUES ('10', '1', '5');

INSERT INTO errtst_parent(partid, shdata, data) VALUES ('20', '1', '5');

INSERT INTO errtst_parent(partid, shdata, data) VALUES ( '0', '1', '10');

INSERT INTO errtst_parent(partid, shdata, data) VALUES ('10', '1', '10');

INSERT INTO errtst_parent(partid, shdata, data) VALUES ('20', '1', '10');

INSERT INTO errtst_parent(partid, shdata, data) VALUES ( '0', '1', NULL);

INSERT INTO errtst_parent(partid, shdata, data) VALUES ('10', '1', NULL);

INSERT INTO errtst_parent(partid, shdata, data) VALUES ('20', '1', NULL);

INSERT INTO errtst_parent(partid, shdata, data) VALUES ( '0', '5', '5');

INSERT INTO errtst_parent(partid, shdata, data) VALUES ('10', '5', '5');

INSERT INTO errtst_parent(partid, shdata, data) VALUES ('20', '5', '5');

BEGIN;

UPDATE errtst_parent SET data = data + 1 WHERE partid = 0;

UPDATE errtst_parent SET data = data + 1 WHERE partid = 10;

UPDATE errtst_parent SET data = data + 1 WHERE partid = 20;

ROLLBACK;

UPDATE errtst_parent SET data = data + 10 WHERE partid = 0;

UPDATE errtst_parent SET data = data + 10 WHERE partid = 10;

UPDATE errtst_parent SET data = data + 10 WHERE partid = 20;

BEGIN;

UPDATE errtst_child_fastdef SET partid = 1 WHERE partid = 0;

UPDATE errtst_child_plaindef SET partid = 11 WHERE partid = 10;

UPDATE errtst_child_reorder SET partid = 21 WHERE partid = 20;

ROLLBACK;

UPDATE errtst_child_fastdef SET partid = partid + 10 WHERE partid = 0;

UPDATE errtst_child_plaindef SET partid = partid + 10 WHERE partid = 10;

UPDATE errtst_child_reorder SET partid = partid + 10 WHERE partid = 20;

BEGIN;

UPDATE errtst_parent SET partid = 10, data = data + 1 WHERE partid = 0;

UPDATE errtst_parent SET partid = 20, data = data + 1 WHERE partid = 10;

UPDATE errtst_parent SET partid = 0, data = data + 1 WHERE partid = 20;

ROLLBACK;

UPDATE errtst_parent SET partid = 10, data = data + 10 WHERE partid = 0;

UPDATE errtst_parent SET partid = 20, data = data + 10 WHERE partid = 10;

UPDATE errtst_parent SET partid = 0, data = data + 10 WHERE partid = 20;

UPDATE errtst_parent SET partid = 30, data = data + 10 WHERE partid = 20;

DROP TABLE errtst_parent;

create table tuplesest_parted (a int, b int, c float) partition by range(a);

create table tuplesest_parted1 partition of tuplesest_parted for values from (0) to (100);

create table tuplesest_parted2 partition of tuplesest_parted for values from (100) to (200);

create table tuplesest_tab (a int, b int);

insert into tuplesest_parted select i%200, i%300, i%400 from generate_series(1, 1000)i;

insert into tuplesest_tab select i, i from generate_series(1, 100)i;

analyze tuplesest_parted;

analyze tuplesest_tab;

select * from tuplesest_tab join
  (select b from tuplesest_parted where c < 100 group by b) sub
  on tuplesest_tab.a = sub.b;

drop table tuplesest_parted;

drop table tuplesest_tab;
