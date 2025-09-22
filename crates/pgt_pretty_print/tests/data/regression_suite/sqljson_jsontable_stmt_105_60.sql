SELECT * FROM JSON_TABLE(jsonb '1', '$' COLUMNS (a int omit quotes true on error));
