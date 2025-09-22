CREATE VIEW json_table_view9 AS SELECT * from JSON_TABLE('"a"', '$' COLUMNS (a text PATH '$') ERROR ON ERROR);
