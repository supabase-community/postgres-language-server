select * from int8_tbl t1
  left join
    (select coalesce(t2.q1 + x, 0) from int8_tbl t2,
       lateral (select t3.q1 as x from int8_tbl t3,
                  lateral (select t2.q1, t3.q1 offset 0) s))
  on true;
