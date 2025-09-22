SELECT JSON_VALUE(jsonb 'null', '$' RETURNING sqljsonb_int_not_null);
