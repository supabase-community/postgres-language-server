SELECT * FROM
  json_populate_recordset(null::record, '[]') AS (x int, y int);
