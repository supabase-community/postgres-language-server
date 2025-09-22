CREATE VIEW json_table_view8 AS SELECT * from JSON_TABLE('"a"', '$' COLUMNS (a text PATH '$') EMPTY ON ERROR);
