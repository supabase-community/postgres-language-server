SELECT * FROM tbl_gist where c4 <@ box(point(1,1),point(10,10));
