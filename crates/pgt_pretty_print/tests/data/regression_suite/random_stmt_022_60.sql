SELECT (count(*) FILTER (WHERE r < -2104533975)) > 0 AS has_small,
       (count(*) FILTER (WHERE r > 2104533974)) > 0 AS has_large
FROM (SELECT random(-2147483648, 2147483647) r FROM generate_series(1, 2000)) ss;
