SELECT * FROM JSON_TABLE(NULL::jsonb, '$' COLUMNS (foo int)) bar;
