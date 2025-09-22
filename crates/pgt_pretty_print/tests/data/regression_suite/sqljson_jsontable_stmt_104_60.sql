SELECT * FROM JSON_TABLE(jsonb '1', '$' COLUMNS (a int true on empty));
