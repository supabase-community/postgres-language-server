SELECT * FROM JSON_TABLE(jsonb '{"d1": "foo"}', '$'
    COLUMNS (js1 oid[] PATH '$.d2' DEFAULT '{1}'::int[]::oid[] ON EMPTY));
