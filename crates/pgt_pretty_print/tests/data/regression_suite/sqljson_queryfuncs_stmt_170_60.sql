SELECT JSON_QUERY(jsonb '"123.1"', '$' RETURNING float8 error on error);
