select * from tenk1 t1
    left join lateral
      (select t1.unique1 as x, * from tenk2 t2 order by 1) t2
    on true
where t1.two > t2.two;
