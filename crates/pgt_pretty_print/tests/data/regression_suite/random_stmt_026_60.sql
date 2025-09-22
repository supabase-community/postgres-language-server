SELECT count(*) FILTER (WHERE r < -1.5 OR r > 1.5) AS out_of_range,
       (count(*) FILTER (WHERE r < -1.47)) > 0 AS has_small,
       (count(*) FILTER (WHERE r > 1.47)) > 0 AS has_large
FROM (SELECT random(-1.500000000000000, 1.500000000000000) r
      FROM generate_series(1, 2000)) ss;
