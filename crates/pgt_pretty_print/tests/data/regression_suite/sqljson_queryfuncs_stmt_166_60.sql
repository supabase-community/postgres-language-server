SELECT JSON_QUERY(jsonb '"123.1"', '$' RETURNING int8 error on error);
