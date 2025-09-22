select jsonb_path_query('{"a": {"b": 1}}', 'lax $.**{1 to last}');
