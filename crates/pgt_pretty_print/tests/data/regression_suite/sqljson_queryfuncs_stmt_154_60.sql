SELECT JSON_QUERY(jsonb '[1,2]', '$' RETURNING char(10));
