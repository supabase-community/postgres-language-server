SELECT *
FROM JSON_TABLE(
  jsonb '{"outer": [{"items": [1, null]}]}', '$.outer[*]'
  COLUMNS (
    outer_row_id FOR ORDINALITY,
    NESTED PATH '$.items[*]'
    COLUMNS (
      idx FOR ORDINALITY,
      value int PATH '$' DEFAULT 0 ON EMPTY,
      raw json PATH '$' WITH WRAPPER KEEP QUOTES,
      cond text PATH '$' DEFAULT 'err' ON ERROR
    )
  )
  EMPTY ARRAY ON ERROR
);
