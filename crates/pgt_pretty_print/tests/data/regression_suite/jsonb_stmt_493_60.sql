SELECT * FROM
  jsonb_populate_recordset(null::record, '[]') AS (x int, y int);
