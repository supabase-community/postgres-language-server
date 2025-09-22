select count(*)
  from (values (point(5,5)),(point(8,8)),(point(12,12))) v(p)
 where exists(select * from spgist_box_tbl b where b.b && box(v.p,v.p));
