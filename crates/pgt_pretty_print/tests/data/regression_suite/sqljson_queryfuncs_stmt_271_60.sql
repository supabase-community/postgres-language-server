SELECT JSON_EXISTS(jsonb '{"a": 123}', '$' || '.' || 'a');
