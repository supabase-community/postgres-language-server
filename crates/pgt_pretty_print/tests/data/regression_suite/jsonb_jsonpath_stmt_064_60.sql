select jsonb_path_query('{"a": 12, "b": {"a": 13}}', '$.b');
