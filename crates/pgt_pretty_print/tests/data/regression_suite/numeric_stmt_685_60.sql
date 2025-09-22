SELECT oper, low, high, cnt, width_bucket(oper, low, high, cnt)
FROM
  (SELECT 1.797e+308::float8 AS big, 5e-324::float8 AS tiny) as v,
  LATERAL (VALUES
    (10.5::float8, -big, big, 1),
    (10.5::float8, -big, big, 2),
    (10.5::float8, -big, big, 3),
    (big / 4, -big / 2, big / 2, 10),
    (10.5::float8, big, -big, 1),
    (10.5::float8, big, -big, 2),
    (10.5::float8, big, -big, 3),
    (big / 4, big / 2, -big / 2, 10),
    (0, 0, tiny, 4),
    (tiny, 0, tiny, 4),
    (0, 0, 1, 2147483647),
    (1, 1, 0, 2147483647)
  ) as sample(oper, low, high, cnt);
