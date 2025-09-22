SELECT * FROM JSON_TABLE(jsonb'"1.23"', '$.a' COLUMNS (js2 int path '$', js2 int path '$'));
