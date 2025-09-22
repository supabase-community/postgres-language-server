SELECT r, count(*)
FROM (SELECT random(-2147483648, 2147483647) r
      FROM generate_series(1, 1000)) ss
GROUP BY r HAVING count(*) > 2;
