select jsonb_path_query('[]', 'strict $.integer()', silent => true);
