SELECT 1 FROM
  (SELECT ntile(s1.x) OVER () AS c
   FROM (SELECT (SELECT 1) AS x) AS s1) s
WHERE s.c = 1;
