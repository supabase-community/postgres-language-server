CREATE TABLE btg AS SELECT
  i % 10 AS x,
  i % 10 AS y,
  'abc' || i % 10 AS z,
  i AS w
FROM generate_series(1, 100) AS i;
