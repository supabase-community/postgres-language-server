select jsonb_path_query('1', 'strict $[1]', silent => true);
