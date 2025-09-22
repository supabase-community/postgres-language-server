SELECT test_missing_target.b, count(*)
  FROM test_missing_target GROUP BY b ORDER BY b;
