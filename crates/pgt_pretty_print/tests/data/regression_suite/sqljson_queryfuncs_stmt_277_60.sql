SELECT JSON_EXISTS(json '{"a": 123}', '$' || '.' || 'a');
