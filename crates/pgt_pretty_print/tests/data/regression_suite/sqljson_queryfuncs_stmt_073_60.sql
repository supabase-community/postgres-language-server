SELECT JSON_VALUE(jsonb '1', 'strict $.*' DEFAULT 2 ON ERROR);
