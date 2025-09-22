select * from semijoin_unique_tbl t1, semijoin_unique_tbl t2
where (t1.a, t2.a) in (select a+1, b+1 from semijoin_unique_tbl t3)
order by t1.a, t2.a;
