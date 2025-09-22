SELECT * FROM JSON_TABLE(jsonb '"a"', '$' COLUMNS (a jsonb EXISTS PATH '$.a'));
