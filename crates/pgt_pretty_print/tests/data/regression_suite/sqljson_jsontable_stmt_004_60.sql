SELECT * FROM JSON_TABLE(jsonb'"1.23"', '$.a' as js2 COLUMNS (js2 int path '$'));
