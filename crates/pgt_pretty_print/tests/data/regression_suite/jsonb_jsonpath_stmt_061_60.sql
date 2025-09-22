select jsonb_path_query('[]', 'strict $[1]', silent => true);
