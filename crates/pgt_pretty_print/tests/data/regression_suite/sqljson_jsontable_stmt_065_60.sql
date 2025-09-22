SELECT a, a::bool FROM JSON_TABLE(jsonb '"a"', '$' COLUMNS (a dint4 EXISTS PATH '$.a' ));
