SELECT * FROM onek
   WHERE onek.unique1 < 10
   ORDER BY onek.unique1;

SELECT onek.unique1, onek.stringu1 FROM onek
   WHERE onek.unique1 < 20
   ORDER BY unique1 using >;

SELECT onek.unique1, onek.stringu1 FROM onek
   WHERE onek.unique1 > 980
   ORDER BY stringu1 using <;

SELECT onek.unique1, onek.string4 FROM onek
   WHERE onek.unique1 > 980
   ORDER BY string4 using <, unique1 using >;

SELECT onek.unique1, onek.string4 FROM onek
   WHERE onek.unique1 > 980
   ORDER BY string4 using >, unique1 using <;

SELECT onek.unique1, onek.string4 FROM onek
   WHERE onek.unique1 < 20
   ORDER BY unique1 using >, string4 using <;

SELECT onek.unique1, onek.string4 FROM onek
   WHERE onek.unique1 < 20
   ORDER BY unique1 using <, string4 using >;

ANALYZE onek2;

SET enable_seqscan TO off;

SET enable_bitmapscan TO off;

SET enable_sort TO off;

SELECT onek2.* FROM onek2 WHERE onek2.unique1 < 10;

SELECT onek2.unique1, onek2.stringu1 FROM onek2
    WHERE onek2.unique1 < 20
    ORDER BY unique1 using >;

SELECT onek2.unique1, onek2.stringu1 FROM onek2
   WHERE onek2.unique1 > 980;

RESET enable_seqscan;

RESET enable_bitmapscan;

RESET enable_sort;

SELECT p.name, p.age FROM person* p;

SELECT p.name, p.age FROM person* p ORDER BY age using >, name;

select foo from (select 1 offset 0) as foo;

select foo from (select null offset 0) as foo;

select foo from (select 'xyzzy',1,null offset 0) as foo;

select * from onek, (values(147, 'RFAAAA'), (931, 'VJAAAA')) as v (i, j)
    WHERE onek.unique1 = v.i and onek.stringu1 = v.j;

select * from onek,
  (values ((select i from
    (values(10000), (2), (389), (1000), (2000), ((select 10029))) as foo(i)
    order by i asc limit 1))) bar (i)
  where onek.unique1 = bar.i;

select * from onek
    where (unique1,ten) in (values (1,1), (20,0), (99,9), (17,99))
    order by unique1;

VALUES (1,2), (3,4+4), (7,77.7);

VALUES (1,2), (3,4+4), (7,77.7)
UNION ALL
SELECT 2+2, 57
UNION ALL
TABLE int8_tbl;

CREATE TEMP TABLE nocols();

INSERT INTO nocols DEFAULT VALUES;

SELECT * FROM nocols n, LATERAL (VALUES(n.*)) v;

CREATE TEMP TABLE foo (f1 int);

INSERT INTO foo VALUES (42),(3),(10),(7),(null),(null),(1);

SELECT * FROM foo ORDER BY f1;

SELECT * FROM foo ORDER BY f1 ASC;

SELECT * FROM foo ORDER BY f1 NULLS FIRST;

SELECT * FROM foo ORDER BY f1 DESC;

SELECT * FROM foo ORDER BY f1 DESC NULLS LAST;

CREATE INDEX fooi ON foo (f1);

SET enable_sort = false;

SELECT * FROM foo ORDER BY f1;

SELECT * FROM foo ORDER BY f1 NULLS FIRST;

SELECT * FROM foo ORDER BY f1 DESC;

SELECT * FROM foo ORDER BY f1 DESC NULLS LAST;

DROP INDEX fooi;

CREATE INDEX fooi ON foo (f1 DESC);

SELECT * FROM foo ORDER BY f1;

SELECT * FROM foo ORDER BY f1 NULLS FIRST;

SELECT * FROM foo ORDER BY f1 DESC;

SELECT * FROM foo ORDER BY f1 DESC NULLS LAST;

DROP INDEX fooi;

CREATE INDEX fooi ON foo (f1 DESC NULLS LAST);

SELECT * FROM foo ORDER BY f1;

SELECT * FROM foo ORDER BY f1 NULLS FIRST;

SELECT * FROM foo ORDER BY f1 DESC;

SELECT * FROM foo ORDER BY f1 DESC NULLS LAST;

select * from onek2 where unique2 = 11 and stringu1 = 'ATAAAA';

select * from onek2 where unique2 = 11 and stringu1 = 'ATAAAA';

select * from onek2 where unique2 = 11 and stringu1 = 'ATAAAA';

select unique2 from onek2 where unique2 = 11 and stringu1 = 'ATAAAA';

select unique2 from onek2 where unique2 = 11 and stringu1 = 'ATAAAA';

select * from onek2 where unique2 = 11 and stringu1 < 'B';

select * from onek2 where unique2 = 11 and stringu1 < 'B';

select unique2 from onek2 where unique2 = 11 and stringu1 < 'B';

select unique2 from onek2 where unique2 = 11 and stringu1 < 'B';

select unique2 from onek2 where unique2 = 11 and stringu1 < 'B' for update;

select unique2 from onek2 where unique2 = 11 and stringu1 < 'B' for update;

select unique2 from onek2 where unique2 = 11 and stringu1 < 'C';

select unique2 from onek2 where unique2 = 11 and stringu1 < 'C';

SET enable_indexscan TO off;

select unique2 from onek2 where unique2 = 11 and stringu1 < 'B';

select unique2 from onek2 where unique2 = 11 and stringu1 < 'B';

RESET enable_indexscan;

select unique1, unique2 from onek2
  where (unique2 = 11 or unique1 = 0) and stringu1 < 'B';

select unique1, unique2 from onek2
  where (unique2 = 11 or unique1 = 0) and stringu1 < 'B';

select unique1, unique2 from onek2
  where (unique2 = 11 and stringu1 < 'B') or unique1 = 0;

select unique1, unique2 from onek2
  where (unique2 = 11 and stringu1 < 'B') or unique1 = 0;

SELECT 1 AS x ORDER BY x;

create function sillysrf(int) returns setof int as
  'values (1),(10),(2),($1)' language sql immutable;

select sillysrf(42);

select sillysrf(-1) order by 1;

drop function sillysrf(int);

select * from (values (2),(null),(1)) v(k) where k = k order by k;

select * from (values (2),(null),(1)) v(k) where k = k;

create table list_parted_tbl (a int,b int) partition by list (a);

create table list_parted_tbl1 partition of list_parted_tbl
  for values in (1) partition by list(b);

select * from list_parted_tbl;

drop table list_parted_tbl;
