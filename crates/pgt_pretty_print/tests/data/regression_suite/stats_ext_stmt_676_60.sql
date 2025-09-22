INSERT INTO expr_stats SELECT mod(i,10), fipshash(mod(i,10)::text), fipshash(mod(i,10)::text) FROM generate_series(1,1000) s(i);
