SELECT JSON_EXISTS(jsonb '{"a": 1, "b": 2}', '$.a.b');
