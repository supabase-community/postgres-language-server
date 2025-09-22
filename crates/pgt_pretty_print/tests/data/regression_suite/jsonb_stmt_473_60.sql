SELECT * FROM
  jsonb_populate_record(null::record, '{"x": 776}') AS (x int, y int);
