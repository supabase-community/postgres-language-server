select jsonb_path_query('[]', 'strict $.double()', silent => true);
