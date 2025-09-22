select jsonb_path_query('{"a": [1, 2]}', 'lax $.a * 3');
