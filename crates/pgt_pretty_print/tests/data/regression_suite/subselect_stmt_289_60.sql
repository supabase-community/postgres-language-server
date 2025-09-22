select t1.q1, x from
  int8_tbl t1 left join
  (int8_tbl t2 left join
   lateral (select t2.q1+1 as x, * from int8_tbl t3) t3 on t2.q2 = t3.q2)
  on t1.q2 = t2.q2
order by 1, 2;
