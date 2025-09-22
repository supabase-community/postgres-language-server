SELECT JSON_VALUE(jsonb '1', 'lax $.a' DEFAULT '2' ON EMPTY);
