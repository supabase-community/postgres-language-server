SELECT r, count(*)
FROM (SELECT random_normal(10, 0) r FROM generate_series(1, 100)) ss
GROUP BY r;
