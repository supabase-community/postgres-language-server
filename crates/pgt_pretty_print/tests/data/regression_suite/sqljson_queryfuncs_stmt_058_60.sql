SELECT JSON_VALUE(jsonb '1',  '$.a' RETURNING sqljsonb_int_not_null DEFAULT 2 ON EMPTY ERROR ON ERROR);
