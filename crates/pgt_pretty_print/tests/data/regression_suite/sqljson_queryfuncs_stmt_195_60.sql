SELECT JSON_QUERY(jsonb '{"a": 1}', '$.b' RETURNING sqljsonb_int_not_null ERROR ON EMPTY ERROR ON ERROR);
