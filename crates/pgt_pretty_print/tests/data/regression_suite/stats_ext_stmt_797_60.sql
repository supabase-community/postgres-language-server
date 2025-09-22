CREATE TABLE sb_1 AS
  SELECT gs % 10 AS x, gs % 10 AS y, gs % 10 AS z
  FROM generate_series(1, 1e4) AS gs;
