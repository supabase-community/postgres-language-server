select t1.q1, x from
  int8_tbl t1 left join
  (int8_tbl t2 left join
   lateral (select t2.q2 as x, * from int8_tbl t3) ss on t2.q2 = ss.q1)
  on t1.q1 = t2.q1
order by 1, 2;
