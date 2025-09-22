SELECT JSON_EXISTS(jsonb '1', 'strict $.a' ERROR ON ERROR);
