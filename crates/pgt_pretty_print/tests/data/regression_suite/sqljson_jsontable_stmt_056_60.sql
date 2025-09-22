SELECT * FROM JSON_TABLE(jsonb '"a"', '$' COLUMNS (a int8 EXISTS PATH '$.a'));
