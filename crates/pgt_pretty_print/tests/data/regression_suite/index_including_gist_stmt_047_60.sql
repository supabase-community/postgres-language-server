INSERT INTO tbl_gist SELECT x, 2*x, 3*x, box(point(3*x,2*x),point(3*x+1,2*x+1)) FROM generate_series(1,10) AS x;
