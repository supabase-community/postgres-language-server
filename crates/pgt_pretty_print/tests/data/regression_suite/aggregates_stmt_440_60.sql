CREATE TABLE group_agg_pk AS SELECT
  i % 10 AS x,
  i % 2 AS y,
  i % 2 AS z,
  2 AS w,
  i % 10 AS f
FROM generate_series(1,100) AS i;
