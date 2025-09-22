select p from gist_tbl where p <@ box(point(0,0), point(0.5, 0.5));
