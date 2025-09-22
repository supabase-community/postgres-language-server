SELECT * FROM JSON_TABLE(jsonb'"1.23"', 'strict $.a' COLUMNS (js2 int PATH '$'));
