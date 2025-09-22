SELECT * FROM
  jsonb_populate_recordset(null::record, '[{"x": 776}]') AS (x int, y int);
