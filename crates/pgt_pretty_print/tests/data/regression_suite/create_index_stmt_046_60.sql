SELECT count(*) FROM point_tbl WHERE box '(0,0,100,100)' @> f1;
