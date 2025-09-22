SELECT JSON_EXISTS(jsonb '[1, "aaa", {"a": 1}]', 'strict $.a');
