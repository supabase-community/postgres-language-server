SELECT count(*) FILTER (WHERE r < -1500000000 OR r > 1500000000) AS out_of_range,
       (count(*) FILTER (WHERE r < -1470000000)) > 0 AS has_small,
       (count(*) FILTER (WHERE r > 1470000000)) > 0 AS has_large
FROM (SELECT random(-1500000000, 1500000000) r FROM generate_series(1, 2000)) ss;
