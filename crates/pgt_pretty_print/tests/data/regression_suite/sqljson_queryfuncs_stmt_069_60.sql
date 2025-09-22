SELECT JSON_VALUE(jsonb '1', 'strict $.a' ERROR ON ERROR);
