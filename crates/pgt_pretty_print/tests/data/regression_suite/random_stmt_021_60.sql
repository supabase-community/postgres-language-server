SELECT r, count(*)
FROM (SELECT random_normal(0, 1 - 1e-15) r
      FROM generate_series(1, 1000)) ss
GROUP BY r HAVING count(*) > 1;
