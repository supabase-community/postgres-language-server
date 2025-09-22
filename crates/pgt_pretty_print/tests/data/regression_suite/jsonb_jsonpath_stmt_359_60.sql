select jsonb_path_query('[]', 'strict $.boolean()', silent => true);
