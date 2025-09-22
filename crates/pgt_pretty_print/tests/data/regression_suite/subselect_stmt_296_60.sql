select ss2.* from
  int8_tbl t1 left join
  (int8_tbl t2 left join
   (select coalesce(q1, q1) as x, * from int8_tbl t3) ss1 on t2.q1 = ss1.q2 left join
   lateral (select ss1.x as y, * from int8_tbl t4) ss2 on t2.q2 = ss2.q1)
  on t1.q2 = ss2.q1
order by 1, 2, 3;
