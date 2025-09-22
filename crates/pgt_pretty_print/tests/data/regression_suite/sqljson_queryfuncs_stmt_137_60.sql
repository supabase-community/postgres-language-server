SELECT JSON_QUERY(jsonb '[]', '$[*]' EMPTY ON EMPTY);
