select explain_filter('explain (verbose) select * from t1 where pg_temp.mysin(f1) < 0.5');
