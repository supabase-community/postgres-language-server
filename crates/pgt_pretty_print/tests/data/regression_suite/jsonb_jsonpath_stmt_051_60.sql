select jsonb_path_query('[]', 'strict $.a', silent => true);
