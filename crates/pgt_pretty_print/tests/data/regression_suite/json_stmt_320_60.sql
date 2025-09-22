SELECT * FROM
  json_populate_recordset(null::record, '[{"x": 776}]') AS (x int, y int);
