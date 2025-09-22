select jsonb_path_query('{"a": [2]}', 'lax $.a + 3');
