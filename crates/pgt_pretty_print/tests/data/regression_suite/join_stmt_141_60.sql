select * from int8_tbl t1
    left join int8_tbl t2 on true
    left join lateral
      (select t2.q1 from int8_tbl t3) s
      on t2.q1 = 1;
