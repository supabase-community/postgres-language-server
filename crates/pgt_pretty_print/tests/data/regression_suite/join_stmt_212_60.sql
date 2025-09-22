select * from tbl_rs t1 join
  lateral (select * from tbl_rs t2 where t2.a in
            (select t1.a+t3.a from tbl_rs t3) and t2.a < 5)
  on true;
