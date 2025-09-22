select jsonb_path_query('[]', 'strict $.string()', silent => true);
