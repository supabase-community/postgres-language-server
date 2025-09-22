SELECT JSON_QUERY(jsonb '123', '$' RETURNING queryfuncs_char2_chk ERROR ON ERROR);
