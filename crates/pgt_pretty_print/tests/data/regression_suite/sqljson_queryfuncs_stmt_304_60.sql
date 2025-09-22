SELECT JSON_VALUE(jsonb '1234', '$' RETURNING queryfuncs_d_varbit3  DEFAULT '01' ON ERROR);
