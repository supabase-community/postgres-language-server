select * from int8_tbl t1 left join
    (int8_tbl t2 left join int8_tbl t3 full join int8_tbl t4 on false on false)
    left join int8_tbl t5 on t2.q1 = t5.q1
on t2.q2 = 123;
