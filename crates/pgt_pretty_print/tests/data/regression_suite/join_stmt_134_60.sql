select * from int4_tbl t1
  left join (int4_tbl t2 left join int4_tbl t3 on t2.f1 > 0) on t2.f1 > 1
  left join int4_tbl t4 on t2.f1 > 2 and t3.f1 > 3
where t1.f1 = coalesce(t2.f1, 1);
