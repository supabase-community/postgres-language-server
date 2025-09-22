SELECT r, count(*)
FROM (SELECT random_normal(-9223372036854775808, 9223372036854775807) r
      FROM generate_series(1, 1000)) ss
GROUP BY r HAVING count(*) > 1;
