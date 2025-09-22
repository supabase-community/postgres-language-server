SELECT JSON_QUERY(jsonb '"123.1"', '$' RETURNING int2 error on error);
