INSERT INTO brin_test_multi_2
SELECT v::uuid FROM (SELECT row_number() OVER (ORDER BY v) c, v FROM (SELECT fipshash((i/13)::text) AS v FROM generate_series(1,1000) s(i)) foo) bar ORDER BY c + 25 * random();
