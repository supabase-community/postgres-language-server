SELECT JSON_OBJECTAGG(mod(i,100): (i)::text FORMAT JSON WITH UNIQUE)
FROM generate_series(0, 199) i;
