SELECT * FROM JSON_TABLE(jsonb '"a"', '$' COLUMNS (a int2 EXISTS PATH '$.a'));
