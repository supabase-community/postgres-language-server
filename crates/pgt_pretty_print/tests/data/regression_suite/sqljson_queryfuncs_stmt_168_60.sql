SELECT JSON_QUERY(jsonb '"123.1"', '$' RETURNING numeric error on error);
