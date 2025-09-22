select jsonb_path_query('1', 'strict $[*]', silent => true);
