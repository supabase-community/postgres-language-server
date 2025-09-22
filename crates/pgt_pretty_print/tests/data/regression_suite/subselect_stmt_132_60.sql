select * from unique_tbl_p t1, unique_tbl_p t2
where (t1.a, t2.a) in (select a, a from unique_tbl_p t3)
order by t1.a, t2.a;
