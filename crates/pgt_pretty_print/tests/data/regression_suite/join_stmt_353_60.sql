select *
from int8_tbl i8
  inner join
    (select (select true) as x
       from int4_tbl i4, lateral (select i4.f1 as y limit 1) ss1
       where i4.f1 = 0) ss2 on true
  right join (select false as z) ss3 on true,
  lateral (select i8.q2 as q2l where x limit 1) ss4
where i8.q2 = 123;
