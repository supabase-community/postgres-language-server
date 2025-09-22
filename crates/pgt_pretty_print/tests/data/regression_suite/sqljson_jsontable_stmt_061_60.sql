SELECT * FROM JSON_TABLE(jsonb '"a"', '$' COLUMNS (a json EXISTS PATH '$.a'));
