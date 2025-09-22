SELECT * FROM JSON_TABLE(jsonb '"a"', '$' COLUMNS (a float4 EXISTS PATH '$.a'));
