INSERT INTO brin_timestamp_test
SELECT '294276-12-01 00:00:01'::timestamptz + (i || ' seconds')::interval
  FROM generate_series(1,30) s(i);
