SELECT JSON_EXISTS(jsonb '{"a": {"b": 1}}', '$.a.b');
