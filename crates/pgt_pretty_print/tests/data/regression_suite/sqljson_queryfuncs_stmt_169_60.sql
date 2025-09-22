SELECT JSON_QUERY(jsonb '"123.1"', '$' RETURNING real error on error);
