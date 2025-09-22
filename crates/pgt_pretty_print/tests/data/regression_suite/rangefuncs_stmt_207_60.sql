SELECT * FROM (VALUES (1),(2),(3)) v(r), ROWS FROM( rngfunc_sql(11,11), rngfunc_mat(10+r,13) );
