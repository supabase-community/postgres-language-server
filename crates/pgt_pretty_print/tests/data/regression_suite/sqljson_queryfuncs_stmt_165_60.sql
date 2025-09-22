SELECT JSON_QUERY(jsonb '"123.1"', '$' RETURNING int4 error on error);
