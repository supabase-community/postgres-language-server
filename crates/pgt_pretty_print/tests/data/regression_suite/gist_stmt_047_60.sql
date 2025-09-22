select p from gist_tbl where circle(p,1) @> circle(point(0,0),0.95);
