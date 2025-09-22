select jsonb_path_query('{"a": [2, 3, 4]}', 'lax -$.a');
