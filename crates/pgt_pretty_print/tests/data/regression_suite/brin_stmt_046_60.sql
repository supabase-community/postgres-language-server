WITH rand_value AS (SELECT string_agg(fipshash(i::text),'') AS val FROM generate_series(1,60) s(i))
INSERT INTO brintest_3
SELECT val, val, val, val FROM rand_value;
