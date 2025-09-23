SELECT 1 AS one WHERE 1 IN (SELECT 1);

SELECT 1 AS zero WHERE 1 NOT IN (SELECT 1);

SELECT 1 AS zero WHERE 1 IN (SELECT 2);

SELECT * FROM (SELECT 1 AS x) ss;

SELECT * FROM ((SELECT 1 AS x)) ss;

SELECT * FROM ((SELECT 1 AS x)), ((SELECT * FROM ((SELECT 2 AS y))));

(SELECT 2) UNION SELECT 2;

((SELECT 2)) UNION SELECT 2;

SELECT ((SELECT 2) UNION SELECT 2);

SELECT (((SELECT 2)) UNION SELECT 2);

SELECT (SELECT ARRAY[1,2,3])[1];

SELECT ((SELECT ARRAY[1,2,3]))[2];

SELECT (((SELECT ARRAY[1,2,3])))[3];

CREATE TABLE SUBSELECT_TBL (
  f1 integer,
  f2 integer,
  f3 float
);

INSERT INTO SUBSELECT_TBL VALUES (1, 2, 3);

INSERT INTO SUBSELECT_TBL VALUES (2, 3, 4);

INSERT INTO SUBSELECT_TBL VALUES (3, 4, 5);

INSERT INTO SUBSELECT_TBL VALUES (1, 1, 1);

INSERT INTO SUBSELECT_TBL VALUES (2, 2, 2);

INSERT INTO SUBSELECT_TBL VALUES (3, 3, 3);

INSERT INTO SUBSELECT_TBL VALUES (6, 7, 8);

INSERT INTO SUBSELECT_TBL VALUES (8, 9, NULL);

SELECT * FROM SUBSELECT_TBL;

SELECT f1 AS "Constant Select" FROM SUBSELECT_TBL
  WHERE f1 IN (SELECT 1);

SELECT f1 AS "Uncorrelated Field" FROM SUBSELECT_TBL
  WHERE f1 IN (SELECT f2 FROM SUBSELECT_TBL);

SELECT f1 AS "Uncorrelated Field" FROM SUBSELECT_TBL
  WHERE f1 IN (SELECT f2 FROM SUBSELECT_TBL WHERE
    f2 IN (SELECT f1 FROM SUBSELECT_TBL));

SELECT f1, f2
  FROM SUBSELECT_TBL
  WHERE (f1, f2) NOT IN (SELECT f2, CAST(f3 AS int4) FROM SUBSELECT_TBL
                         WHERE f3 IS NOT NULL);

SELECT f1 AS "Correlated Field", f2 AS "Second Field"
  FROM SUBSELECT_TBL upper
  WHERE f1 IN (SELECT f2 FROM SUBSELECT_TBL WHERE f1 = upper.f1);

SELECT f1 AS "Correlated Field", f3 AS "Second Field"
  FROM SUBSELECT_TBL upper
  WHERE f1 IN
    (SELECT f2 FROM SUBSELECT_TBL WHERE CAST(upper.f2 AS float) = f3);

SELECT f1 AS "Correlated Field", f3 AS "Second Field"
  FROM SUBSELECT_TBL upper
  WHERE f3 IN (SELECT upper.f1 + f2 FROM SUBSELECT_TBL
               WHERE f2 = CAST(f3 AS integer));

SELECT f1 AS "Correlated Field"
  FROM SUBSELECT_TBL
  WHERE (f1, f2) IN (SELECT f2, CAST(f3 AS int4) FROM SUBSELECT_TBL
                     WHERE f3 IS NOT NULL);

SELECT ROW(1, 2) = (SELECT f1, f2) AS eq FROM SUBSELECT_TBL;

SELECT ROW(1, 2) = (SELECT f1, f2) AS eq FROM SUBSELECT_TBL;

SELECT ROW(1, 2) = (SELECT 3, 4) AS eq FROM SUBSELECT_TBL;

SELECT ROW(1, 2) = (SELECT 3, 4) AS eq FROM SUBSELECT_TBL;

SELECT ROW(1, 2) = (SELECT f1, f2 FROM SUBSELECT_TBL);

SELECT count FROM (SELECT COUNT(DISTINCT name) FROM road);

SELECT COUNT(*) FROM (SELECT DISTINCT name FROM road);

SELECT * FROM (SELECT * FROM int4_tbl), (VALUES (123456)) WHERE f1 = column1;

CREATE VIEW view_unnamed_ss AS
SELECT * FROM (SELECT * FROM (SELECT abs(f1) AS a1 FROM int4_tbl)),
              (SELECT * FROM int8_tbl)
  WHERE a1 < 10 AND q1 > a1 ORDER BY q1, q2;

SELECT * FROM view_unnamed_ss;

DROP VIEW view_unnamed_ss;

CREATE VIEW view_unnamed_ss_locking AS
SELECT * FROM (SELECT * FROM int4_tbl), int8_tbl AS unnamed_subquery
  WHERE f1 = q1
  FOR UPDATE OF unnamed_subquery;

DROP VIEW view_unnamed_ss_locking;

SELECT ss.f1 AS "Correlated Field", ss.f3 AS "Second Field"
  FROM SUBSELECT_TBL ss
  WHERE f1 NOT IN (SELECT f1+1 FROM INT4_TBL
                   WHERE f1 != ss.f1 AND f1 < 2147483647);

select q1, float8(count(*)) / (select count(*) from int8_tbl)
from int8_tbl group by q1 order by q1;

SELECT *, pg_typeof(f1) FROM
  (SELECT 'foo' AS f1 FROM generate_series(1,3)) ss ORDER BY 1;

select '42' union all select '43';

select '42' union all select 43;

select 1 = all (select (select 1));

select 1 = all (select (select 1));

select * from int4_tbl o where exists
  (select 1 from int4_tbl i where i.f1=o.f1 limit null);

select * from int4_tbl o where not exists
  (select 1 from int4_tbl i where i.f1=o.f1 limit 1);

select * from int4_tbl o where exists
  (select 1 from int4_tbl i where i.f1=o.f1 limit 0);

select count(*) from
  (select 1 from tenk1 a
   where unique1 IN (select hundred from tenk1 b)) ss;

select count(distinct ss.ten) from
  (select ten from tenk1 a
   where unique1 IN (select hundred from tenk1 b)) ss;

select count(*) from
  (select 1 from tenk1 a
   where unique1 IN (select distinct hundred from tenk1 b)) ss;

select count(distinct ss.ten) from
  (select ten from tenk1 a
   where unique1 IN (select distinct hundred from tenk1 b)) ss;

CREATE TEMP TABLE foo (id integer);

CREATE TEMP TABLE bar (id1 integer, id2 integer);

INSERT INTO foo VALUES (1);

INSERT INTO bar VALUES (1, 1);

INSERT INTO bar VALUES (2, 2);

INSERT INTO bar VALUES (3, 1);

SELECT * FROM foo WHERE id IN
    (SELECT id2 FROM (SELECT DISTINCT id1, id2 FROM bar) AS s);

SELECT * FROM foo WHERE id IN
    (SELECT id2 FROM (SELECT id1,id2 FROM bar GROUP BY id1,id2) AS s);

SELECT * FROM foo WHERE id IN
    (SELECT id2 FROM (SELECT id1, id2 FROM bar UNION
                      SELECT id1, id2 FROM bar) AS s);

SELECT * FROM foo WHERE id IN
    (SELECT id2 FROM (SELECT DISTINCT ON (id2) id1, id2 FROM bar) AS s);

SELECT * FROM foo WHERE id IN
    (SELECT id2 FROM (SELECT id2 FROM bar GROUP BY id2) AS s);

SELECT * FROM foo WHERE id IN
    (SELECT id2 FROM (SELECT id2 FROM bar UNION
                      SELECT id2 FROM bar) AS s);

CREATE TABLE orderstest (
    approver_ref integer,
    po_ref integer,
    ordercanceled boolean
);

INSERT INTO orderstest VALUES (1, 1, false);

INSERT INTO orderstest VALUES (66, 5, false);

INSERT INTO orderstest VALUES (66, 6, false);

INSERT INTO orderstest VALUES (66, 7, false);

INSERT INTO orderstest VALUES (66, 1, true);

INSERT INTO orderstest VALUES (66, 8, false);

INSERT INTO orderstest VALUES (66, 1, false);

INSERT INTO orderstest VALUES (77, 1, false);

INSERT INTO orderstest VALUES (1, 1, false);

INSERT INTO orderstest VALUES (66, 1, false);

INSERT INTO orderstest VALUES (1, 1, false);

CREATE VIEW orders_view AS
SELECT *,
(SELECT CASE
   WHEN ord.approver_ref=1 THEN '---' ELSE 'Approved'
 END) AS "Approved",
(SELECT CASE
 WHEN ord.ordercanceled
 THEN 'Canceled'
 ELSE
  (SELECT CASE
		WHEN ord.po_ref=1
		THEN
		 (SELECT CASE
				WHEN ord.approver_ref=1
				THEN '---'
				ELSE 'Approved'
			END)
		ELSE 'PO'
	END)
END) AS "Status",
(CASE
 WHEN ord.ordercanceled
 THEN 'Canceled'
 ELSE
  (CASE
		WHEN ord.po_ref=1
		THEN
		 (CASE
				WHEN ord.approver_ref=1
				THEN '---'
				ELSE 'Approved'
			END)
		ELSE 'PO'
	END)
END) AS "Status_OK"
FROM orderstest ord;

SELECT * FROM orders_view;

DROP TABLE orderstest cascade;

create temp table parts (
    partnum     text,
    cost        float8
);

create temp table shipped (
    ttype       char(2),
    ordnum      int4,
    partnum     text,
    value       float8
);

create temp view shipped_view as
    select * from shipped where ttype = 'wt';

create rule shipped_view_insert as on insert to shipped_view do instead
    insert into shipped values('wt', new.ordnum, new.partnum, new.value);

insert into parts (partnum, cost) values (1, 1234.56);

insert into shipped_view (ordnum, partnum, value)
    values (0, 1, (select cost from parts where partnum = '1'));

select * from shipped_view;

create rule shipped_view_update as on update to shipped_view do instead
    update shipped set partnum = new.partnum, value = new.value
        where ttype = new.ttype and ordnum = new.ordnum;

update shipped_view set value = 11
    from int4_tbl a join int4_tbl b
      on (a.f1 = (select f1 from int4_tbl c where c.f1=b.f1))
    where ordnum = a.f1;

select * from shipped_view;

select f1, ss1 as relabel from
    (select *, (select sum(f1) from int4_tbl b where f1 >= a.f1) as ss1
     from int4_tbl a) ss;

select * from (
  select max(unique1) from tenk1 as a
  where exists (select 1 from tenk1 as b where b.thousand = a.unique2)
) ss;

select * from (
  select min(unique1) from tenk1 as a
  where not exists (select 1 from tenk1 as b where b.unique2 = 10000)
) ss;

create temp table numeric_table (num_col numeric);

insert into numeric_table values (1), (1.000000000000000000001), (2), (3);

create temp table float_table (float_col float8);

insert into float_table values (1), (2), (3);

select * from float_table
  where float_col in (select num_col from numeric_table);

select * from numeric_table
  where num_col in (select float_col from float_table);

create table semijoin_unique_tbl (a int, b int);

insert into semijoin_unique_tbl select i%10, i%10 from generate_series(1,1000)i;

create index on semijoin_unique_tbl(a, b);

analyze semijoin_unique_tbl;

select * from semijoin_unique_tbl t1, semijoin_unique_tbl t2
where (t1.a, t2.a) in (select a, b from semijoin_unique_tbl t3)
order by t1.a, t2.a;

select * from semijoin_unique_tbl t1, semijoin_unique_tbl t2
where (t1.a, t2.a) in (select a+1, b+1 from semijoin_unique_tbl t3)
order by t1.a, t2.a;

set parallel_setup_cost=0;

set parallel_tuple_cost=0;

set min_parallel_table_scan_size=0;

set max_parallel_workers_per_gather=4;

set enable_indexscan to off;

select * from semijoin_unique_tbl t1, semijoin_unique_tbl t2
where (t1.a, t2.a) in (select a, b from semijoin_unique_tbl t3)
order by t1.a, t2.a;

reset enable_indexscan;

reset max_parallel_workers_per_gather;

reset min_parallel_table_scan_size;

reset parallel_tuple_cost;

reset parallel_setup_cost;

drop table semijoin_unique_tbl;

create table unique_tbl_p (a int, b int) partition by range(a);

create table unique_tbl_p1 partition of unique_tbl_p for values from (0) to (5);

create table unique_tbl_p2 partition of unique_tbl_p for values from (5) to (10);

create table unique_tbl_p3 partition of unique_tbl_p for values from (10) to (20);

insert into unique_tbl_p select i%12, i from generate_series(0, 1000)i;

create index on unique_tbl_p1(a);

create index on unique_tbl_p2(a);

create index on unique_tbl_p3(a);

analyze unique_tbl_p;

set enable_partitionwise_join to on;

select * from unique_tbl_p t1, unique_tbl_p t2
where (t1.a, t2.a) in (select a, a from unique_tbl_p t3)
order by t1.a, t2.a;

reset enable_partitionwise_join;

drop table unique_tbl_p;

create temp table ta (id int primary key, val int);

insert into ta values(1,1);

insert into ta values(2,2);

create temp table tb (id int primary key, aval int);

insert into tb values(1,1);

insert into tb values(2,1);

insert into tb values(3,2);

insert into tb values(4,2);

create temp table tc (id int primary key, aid int);

insert into tc values(1,1);

insert into tc values(2,2);

select
  ( select min(tb.id) from tb
    where tb.aval = (select ta.val from ta where ta.id = tc.aid) ) as min_tb_id
from tc;

create temp table t1 (f1 numeric(14,0), f2 varchar(30));

select * from
  (select distinct f1, f2, (select f2 from t1 x where x.f1 = up.f1) as fs
   from t1 up) ss
group by f1,f2,fs;

create temp table table_a(id integer);

insert into table_a values (42);

create temp view view_a as select * from table_a;

select view_a from view_a;

select (select view_a) from view_a;

select (select (select view_a)) from view_a;

select (select (a.*)::text) from view_a a;

select (1 = any(array_agg(f1))) = any (select false) from int4_tbl;

select (1 = any(array_agg(f1))) = any (select false) from int4_tbl;

select q from (select max(f1) from int4_tbl group by f1 order by f1) q;

with q as (select max(f1) from int4_tbl group by f1 order by f1)
  select q from q;

begin;

delete from road
where exists (
  select 1
  from
    int4_tbl cross join
    ( select f1, array(select q1 from int8_tbl) as arr
      from text_tbl ) ss
  where road.name = ss.f1 );

rollback;

select
  (select sq1) as qq1
from
  (select exists(select 1 from int4_tbl where f1 = q2) as sq1, 42 as dummy
   from int8_tbl) sq0
  join
  int4_tbl i4 on dummy = i4.f1;

create temp table upsert(key int4 primary key, val text);

insert into upsert values(1, 'val') on conflict (key) do update set val = 'not seen';

insert into upsert values(1, 'val') on conflict (key) do update set val = 'seen with subselect ' || (select f1 from int4_tbl where f1 != 0 limit 1)::text;

select * from upsert;

with aa as (select 'int4_tbl' u from int4_tbl limit 1)
insert into upsert values (1, 'x'), (999, 'y')
on conflict (key) do update set val = (select u from aa)
returning *;

create temp table outer_7597 (f1 int4, f2 int4);

insert into outer_7597 values (0, 0);

insert into outer_7597 values (1, 0);

insert into outer_7597 values (0, null);

insert into outer_7597 values (1, null);

create temp table inner_7597(c1 int8, c2 int8);

insert into inner_7597 values(0, null);

select * from outer_7597 where (f1, f2) not in (select * from inner_7597);

create temp table outer_text (f1 text, f2 text);

insert into outer_text values ('a', 'a');

insert into outer_text values ('b', 'a');

insert into outer_text values ('a', null);

insert into outer_text values ('b', null);

create temp table inner_text (c1 text, c2 text);

insert into inner_text values ('a', null);

insert into inner_text values ('123', '456');

select * from outer_text where (f1, f2) not in (select * from inner_text);

select 'foo'::text in (select 'bar'::name union all select 'bar'::name);

select 'foo'::text in (select 'bar'::name union all select 'bar'::name);

select row(row(row(1))) = any (select row(row(1)));

select row(row(row(1))) = any (select row(row(1)));

select '1'::text in (select '1'::name union all select '1'::name);

select * from int8_tbl where q1 in (select c1 from inner_text);

begin;

create function bogus_int8_text_eq(int8, text) returns boolean
language sql as 'select $1::text = $2';

create operator = (procedure=bogus_int8_text_eq, leftarg=int8, rightarg=text);

select * from int8_tbl where q1 in (select c1 from inner_text);

select * from int8_tbl where q1 in (select c1 from inner_text);

create or replace function bogus_int8_text_eq(int8, text) returns boolean
language sql as 'select $1::text = $2 and $1::text = $2';

select * from int8_tbl where q1 in (select c1 from inner_text);

select * from int8_tbl where q1 in (select c1 from inner_text);

create or replace function bogus_int8_text_eq(int8, text) returns boolean
language sql as 'select $2 = $1::text';

select * from int8_tbl where q1 in (select c1 from inner_text);

select * from int8_tbl where q1 in (select c1 from inner_text);

rollback;

select count(*) from tenk1 t
where (exists(select 1 from tenk1 k where k.unique1 = t.unique2) or ten < 0);

select count(*) from tenk1 t
where (exists(select 1 from tenk1 k where k.unique1 = t.unique2) or ten < 0);

select count(*) from tenk1 t
where (exists(select 1 from tenk1 k where k.unique1 = t.unique2) or ten < 0)
  and thousand = 1;

select count(*) from tenk1 t
where (exists(select 1 from tenk1 k where k.unique1 = t.unique2) or ten < 0)
  and thousand = 1;

create temp table exists_tbl (c1 int, c2 int, c3 int) partition by list (c1);

create temp table exists_tbl_null partition of exists_tbl for values in (null);

create temp table exists_tbl_def partition of exists_tbl default;

insert into exists_tbl select x, x/2, x+1 from generate_series(0,10) x;

analyze exists_tbl;

select * from exists_tbl t1
  where (exists(select 1 from exists_tbl t2 where t1.c1 = t2.c2) or c3 < 0);

select * from exists_tbl t1
  where (exists(select 1 from exists_tbl t2 where t1.c1 = t2.c2) or c3 < 0);

select a.thousand from tenk1 a, tenk1 b
where a.thousand = b.thousand
  and exists ( select 1 from tenk1 c where b.hundred = c.hundred
                   and not exists ( select 1 from tenk1 d
                                    where a.thousand = d.thousand ) );

select x, x from
    (select (select now()) as x from (values(1),(2)) v(y)) ss;

select x, x from
    (select (select random()) as x from (values(1),(2)) v(y)) ss;

select x, x from
    (select (select now() where y=y) as x from (values(1),(2)) v(y)) ss;

select x, x from
    (select (select random() where y=y) as x from (values(1),(2)) v(y)) ss;

select sum(ss.tst::int) from
  onek o cross join lateral (
  select i.ten in (select f1 from int4_tbl where f1 <= o.hundred) as tst,
         random() as r
  from onek i where i.unique1 = o.unique1 ) ss
where o.ten = 0;

select sum(ss.tst::int) from
  onek o cross join lateral (
  select i.ten in (select f1 from int4_tbl where f1 <= o.hundred) as tst,
         random() as r
  from onek i where i.unique1 = o.unique1 ) ss
where o.ten = 0;

begin;

set local enable_sort = off;

select count(*) from
  onek o cross join lateral (
    select * from onek i1 where i1.unique1 = o.unique1
    except
    select * from onek i2 where i2.unique1 = o.unique2
  ) ss
where o.ten = 1;

select count(*) from
  onek o cross join lateral (
    select * from onek i1 where i1.unique1 = o.unique1
    except
    select * from onek i2 where i2.unique1 = o.unique2
  ) ss
where o.ten = 1;

rollback;

begin;

set local enable_hashagg = off;

select count(*) from
  onek o cross join lateral (
    select * from onek i1 where i1.unique1 = o.unique1
    except
    select * from onek i2 where i2.unique1 = o.unique2
  ) ss
where o.ten = 1;

select count(*) from
  onek o cross join lateral (
    select * from onek i1 where i1.unique1 = o.unique1
    except
    select * from onek i2 where i2.unique1 = o.unique2
  ) ss
where o.ten = 1;

rollback;

select sum(o.four), sum(ss.a) from
  onek o cross join lateral (
    with recursive x(a) as
      (select o.four as a
       union
       select a + 1 from x
       where a < 10)
    select * from x
  ) ss
where o.ten = 1;

select sum(o.four), sum(ss.a) from
  onek o cross join lateral (
    with recursive x(a) as
      (select o.four as a
       union
       select a + 1 from x
       where a < 10)
    select * from x
  ) ss
where o.ten = 1;

create temp table notinouter (a int);

create temp table notininner (b int not null);

insert into notinouter values (null), (1);

select * from notinouter where a not in (select b from notininner);

create temp table nocolumns();

select exists(select * from nocolumns);

select val.x
  from generate_series(1,10) as s(i),
  lateral (
    values ((select s.i + 1)), (s.i + 101)
  ) as val(x)
where s.i < 10 and (select val.x) < 110;

select * from
(values
  (3 not in (select * from (values (1), (2)) ss1)),
  (false)
) ss;

select * from
(values
  (3 not in (select * from (values (1), (2)) ss1)),
  (false)
) ss;

select * from int4_tbl where
  (case when f1 in (select unique1 from tenk1 a) then f1 else null end) in
  (select ten from tenk1 b);

select * from int4_tbl where
  (case when f1 in (select unique1 from tenk1 a) then f1 else null end) in
  (select ten from tenk1 b);

select * from int4_tbl o where (f1, f1) in
  (select f1, generate_series(1,50) / 10 g from int4_tbl i group by f1);

select * from int4_tbl o where (f1, f1) in
  (select f1, generate_series(1,50) / 10 g from int4_tbl i group by f1);

select (select q from
         (select 1,2,3 where f1 > 0
          union all
          select 4,5,6.0 where f1 <= 0
         ) q )
from int4_tbl;

select * from
    int4_tbl i4,
    lateral (
        select i4.f1 > 1 as b, 1 as id
        from (select random() order by 1) as t1
      union all
        select true as b, 2 as id
    ) as t2
where b and f1 >= 0;

select * from
    int4_tbl i4,
    lateral (
        select i4.f1 > 1 as b, 1 as id
        from (select random() order by 1) as t1
      union all
        select true as b, 2 as id
    ) as t2
where b and f1 >= 0;

create temp sequence ts1;

select * from
  (select distinct ten from tenk1) ss
  where ten < 10 + nextval('ts1')
  order by 1;

select nextval('ts1');

create function tattle(x int, y int) returns bool
volatile language plpgsql as $$
begin
  raise notice 'x = %, y = %', x, y;
  return x > y;
end$$;

select * from
  (select 9 as x, unnest(array[1,2,3,11,12,13]) as u) ss
  where tattle(x, 8);

select * from
  (select 9 as x, unnest(array[1,2,3,11,12,13]) as u) ss
  where tattle(x, 8);

alter function tattle(x int, y int) stable;

select * from
  (select 9 as x, unnest(array[1,2,3,11,12,13]) as u) ss
  where tattle(x, 8);

select * from
  (select 9 as x, unnest(array[1,2,3,11,12,13]) as u) ss
  where tattle(x, 8);

select * from
  (select 9 as x, unnest(array[1,2,3,11,12,13]) as u) ss
  where tattle(x, u);

select * from
  (select 9 as x, unnest(array[1,2,3,11,12,13]) as u) ss
  where tattle(x, u);

drop function tattle(x int, y int);

create table sq_limit (pk int primary key, c1 int, c2 int);

insert into sq_limit values
    (1, 1, 1),
    (2, 2, 2),
    (3, 3, 3),
    (4, 4, 4),
    (5, 1, 1),
    (6, 2, 2),
    (7, 3, 3),
    (8, 4, 4);

create function explain_sq_limit() returns setof text language plpgsql as
$$
declare ln text;
begin
    for ln in
        explain (analyze, summary off, timing off, costs off, buffers off)
        select * from (select pk,c2 from sq_limit order by c1,pk) as x limit 3
    loop
        ln := regexp_replace(ln, 'Memory: \S*',  'Memory: xxx');
        return next ln;
    end loop;
end;
$$;

select * from explain_sq_limit();

select * from (select pk,c2 from sq_limit order by c1,pk) as x limit 3;

drop function explain_sq_limit();

drop table sq_limit;

begin;

declare c1 scroll cursor for
 select * from generate_series(1,4) i
  where i <> all (values (2),(3));

move forward all in c1;

fetch backward all in c1;

commit;

begin;

create temp table json_tab (a int);

insert into json_tab values (1);

select * from json_tab t1 left join (select json_array(1, a) from json_tab t2) s on false;

select * from json_tab t1 left join (select json_array(1, a) from json_tab t2) s on false;

rollback;

select tname, attname from (
select relname::information_schema.sql_identifier as tname, * from
  (select * from pg_class c) ss1) ss2
  right join pg_attribute a on a.attrelid = ss2.oid
where tname = 'tenk1' and attnum = 1;

select tname, attname from (
select relname::information_schema.sql_identifier as tname, * from
  (select * from pg_class c) ss1) ss2
  right join pg_attribute a on a.attrelid = ss2.oid
where tname = 'tenk1' and attnum = 1;

select t1.ten, sum(x) from
  tenk1 t1 left join lateral (
    select t1.ten + t2.ten as x, t2.fivethous from tenk1 t2
  ) ss on t1.unique1 = ss.fivethous
group by t1.ten
order by t1.ten;

select t1.ten, sum(x) from
  tenk1 t1 left join lateral (
    select t1.ten + t2.ten as x, t2.fivethous from tenk1 t2
  ) ss on t1.unique1 = ss.fivethous
group by t1.ten
order by t1.ten;

select t1.q1, x from
  int8_tbl t1 left join
  (int8_tbl t2 left join
   lateral (select t2.q1+t3.q1 as x, * from int8_tbl t3) t3 on t2.q2 = t3.q2)
  on t1.q2 = t2.q2
order by 1, 2;

select t1.q1, x from
  int8_tbl t1 left join
  (int8_tbl t2 left join
   lateral (select t2.q1+t3.q1 as x, * from int8_tbl t3) t3 on t2.q2 = t3.q2)
  on t1.q2 = t2.q2
order by 1, 2;

select t1.q1, x from
  int8_tbl t1 left join
  (int8_tbl t2 inner join
   lateral (select t2.q1+1 as x, * from int8_tbl t3) t3 on t2.q2 = t3.q2)
  on t1.q2 = t2.q2
order by 1, 2;

select t1.q1, x from
  int8_tbl t1 left join
  (int8_tbl t2 inner join
   lateral (select t2.q1+1 as x, * from int8_tbl t3) t3 on t2.q2 = t3.q2)
  on t1.q2 = t2.q2
order by 1, 2;

select t1.q1, x from
  int8_tbl t1 left join
  (int8_tbl t2 left join
   lateral (select t2.q1+1 as x, * from int8_tbl t3) t3 on t2.q2 = t3.q2)
  on t1.q2 = t2.q2
order by 1, 2;

select t1.q1, x from
  int8_tbl t1 left join
  (int8_tbl t2 left join
   lateral (select t2.q1+1 as x, * from int8_tbl t3) t3 on t2.q2 = t3.q2)
  on t1.q2 = t2.q2
order by 1, 2;

select t1.q1, x from
  int8_tbl t1 left join
  (int8_tbl t2 inner join
   lateral (select t2.q2 as x, * from int8_tbl t3) ss on t2.q2 = ss.q1)
  on t1.q1 = t2.q1
order by 1, 2;

select t1.q1, x from
  int8_tbl t1 left join
  (int8_tbl t2 inner join
   lateral (select t2.q2 as x, * from int8_tbl t3) ss on t2.q2 = ss.q1)
  on t1.q1 = t2.q1
order by 1, 2;

select t1.q1, x from
  int8_tbl t1 left join
  (int8_tbl t2 left join
   lateral (select t2.q2 as x, * from int8_tbl t3) ss on t2.q2 = ss.q1)
  on t1.q1 = t2.q1
order by 1, 2;

select t1.q1, x from
  int8_tbl t1 left join
  (int8_tbl t2 left join
   lateral (select t2.q2 as x, * from int8_tbl t3) ss on t2.q2 = ss.q1)
  on t1.q1 = t2.q1
order by 1, 2;

select ss2.* from
  int8_tbl t1 left join
  (int8_tbl t2 left join
   (select coalesce(q1, q1) as x, * from int8_tbl t3) ss1 on t2.q1 = ss1.q2 inner join
   lateral (select ss1.x as y, * from int8_tbl t4) ss2 on t2.q2 = ss2.q1)
  on t1.q2 = ss2.q1
order by 1, 2, 3;

select ss2.* from
  int8_tbl t1 left join
  (int8_tbl t2 left join
   (select coalesce(q1, q1) as x, * from int8_tbl t3) ss1 on t2.q1 = ss1.q2 inner join
   lateral (select ss1.x as y, * from int8_tbl t4) ss2 on t2.q2 = ss2.q1)
  on t1.q2 = ss2.q1
order by 1, 2, 3;

select ss2.* from
  int8_tbl t1 left join
  (int8_tbl t2 left join
   (select coalesce(q1, q1) as x, * from int8_tbl t3) ss1 on t2.q1 = ss1.q2 left join
   lateral (select ss1.x as y, * from int8_tbl t4) ss2 on t2.q2 = ss2.q1)
  on t1.q2 = ss2.q1
order by 1, 2, 3;

select ss2.* from
  int8_tbl t1 left join
  (int8_tbl t2 left join
   (select coalesce(q1, q1) as x, * from int8_tbl t3) ss1 on t2.q1 = ss1.q2 left join
   lateral (select ss1.x as y, * from int8_tbl t4) ss2 on t2.q2 = ss2.q1)
  on t1.q2 = ss2.q1
order by 1, 2, 3;

with x as (select * from (select f1 from subselect_tbl) ss)
select * from x where f1 = 1;
