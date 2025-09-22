SELECT JSON_QUERY(jsonb '[{"a": "a", "b": "foo", "t": "aaa", "js": [1, "2", {}], "jb": {"x": [1, "2", {}]}},  {"a": 2}]', '$[0]' RETURNING sqljsonb_rec ERROR ON ERROR);
