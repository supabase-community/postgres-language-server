SELECT * FROM JSON_TABLE(jsonb '1', '$' COLUMNS (a int exists empty object on error));
