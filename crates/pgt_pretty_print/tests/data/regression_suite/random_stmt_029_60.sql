SELECT min(r), max(r), count(r) FROM (
  SELECT DISTINCT random(-0.5, 0.49) r FROM generate_series(1, 2500));
