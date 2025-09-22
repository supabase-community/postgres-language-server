SELECT a, a::bool FROM JSON_TABLE(jsonb '{"a":1}', '$' COLUMNS (a dint4_0 EXISTS PATH '$.b' TRUE ON ERROR));
