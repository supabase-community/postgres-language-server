select * from jsonb_path_query('{"a": 10}', '$ ? (@.a < $value)', '{"value" : 8}');
