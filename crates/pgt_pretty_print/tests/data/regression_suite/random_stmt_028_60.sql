SELECT min(r), max(r), count(r) FROM (
  SELECT DISTINCT random(123000000000, 123000000099) r
  FROM generate_series(1, 2500));
