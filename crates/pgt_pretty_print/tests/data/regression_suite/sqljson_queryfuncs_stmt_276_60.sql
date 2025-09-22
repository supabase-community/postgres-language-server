SELECT JSON_QUERY(jsonb '{"a": 123}', 'error' || ' ' || 'error');
