SELECT JSON_QUERY(jsonb '{"a": 1}', '$.a' RETURNING sqljsonb_int_not_null);
