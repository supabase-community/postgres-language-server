SELECT JSON_QUERY(jsonb '123', '$' RETURNING queryfuncs_char2_chk DEFAULT '1' ON ERROR);
