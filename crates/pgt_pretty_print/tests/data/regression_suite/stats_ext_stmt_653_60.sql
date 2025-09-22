INSERT INTO expr_stats SELECT mod(i,10), mod(i,10), mod(i,10) FROM generate_series(1,1000) s(i);
