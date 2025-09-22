CREATE TABLE sb_2 AS
  SELECT gs % 49 AS x, gs % 51 AS y, gs % 73 AS z, 'abc' || gs AS payload
  FROM generate_series(1, 1e4) AS gs;
