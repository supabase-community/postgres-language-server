select * from onek t1
    left join onek t2 on true
    left join lateral
      (select * from onek t3 where t3.two = t2.two offset 0) s
      on t2.unique1 = 1;
