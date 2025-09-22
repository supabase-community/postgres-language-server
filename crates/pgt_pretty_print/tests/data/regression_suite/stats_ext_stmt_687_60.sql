SELECT c0 FROM ONLY expr_stats_incompatible_test WHERE
(
  upper('x') LIKE ('x'||('[0,1]'::int4range))
  AND
  (c0 IN (0, 1) OR c1)
);
