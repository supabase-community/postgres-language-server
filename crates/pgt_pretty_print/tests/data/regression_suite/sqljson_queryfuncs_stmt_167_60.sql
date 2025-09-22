SELECT JSON_QUERY(jsonb '"123.1"', '$' RETURNING bool error on error);
