select circle(p,1) from gist_tbl
where p <@ box(point(5, 5), point(5.3, 5.3));
