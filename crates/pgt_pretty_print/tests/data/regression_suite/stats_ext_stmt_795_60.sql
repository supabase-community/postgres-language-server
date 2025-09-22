SELECT * FROM check_estimated_rows('
  SELECT * FROM generate_series(1, 1) t1 LEFT JOIN (
    SELECT x FROM grouping_unique t2 GROUP BY x) AS q1
  ON t1.t1 = q1.x;
');
