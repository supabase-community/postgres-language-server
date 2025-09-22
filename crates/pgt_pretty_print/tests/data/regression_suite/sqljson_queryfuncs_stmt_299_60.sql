SELECT JSON_VALUE(jsonb '123', '$' RETURNING queryfuncs_char2_chk DEFAULT 1 ON ERROR);
