SELECT JSON_VALUE(jsonb '1', 'lax $.a' ERROR ON ERROR);
