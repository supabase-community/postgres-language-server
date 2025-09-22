SELECT JSON_VALUE(jsonb '["1"]', '$[*]' RETURNING record);
