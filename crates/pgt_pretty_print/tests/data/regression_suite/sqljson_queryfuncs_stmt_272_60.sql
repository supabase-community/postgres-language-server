SELECT JSON_VALUE(jsonb '{"a": 123}', '$' || '.' || 'a');
