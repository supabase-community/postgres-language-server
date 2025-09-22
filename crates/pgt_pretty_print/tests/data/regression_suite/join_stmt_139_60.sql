select * from int8_tbl t1
    left join int8_tbl t2 on true
    left join lateral
      (select * from int8_tbl t3 where t3.q1 = t2.q1 offset 0) s
      on t2.q1 = 1;
