CREATE VIEW jsonb_table_view2 AS
SELECT *
FROM JSON_TABLE(
  jsonb 'null', 'lax $[*]' PASSING 1 + 2 AS a, json '"foo"' AS "b c"
  COLUMNS (
    "int" int PATH '$',
    "text" text PATH '$',
    js json PATH '$',
    jsb jsonb PATH '$'
  )
  NULL ON ERROR
);
