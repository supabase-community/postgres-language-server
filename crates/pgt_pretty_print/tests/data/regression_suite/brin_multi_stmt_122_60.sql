INSERT INTO brin_timestamp_test
SELECT '4713-01-01 00:00:01 BC'::timestamptz + (i || ' seconds')::interval
  FROM generate_series(1,30) s(i);
