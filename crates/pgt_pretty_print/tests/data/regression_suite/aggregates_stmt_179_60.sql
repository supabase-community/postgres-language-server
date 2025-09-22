select f1, (select distinct min(t1.f1) from int4_tbl t1 where t1.f1 = t0.f1)
  from int4_tbl t0;
