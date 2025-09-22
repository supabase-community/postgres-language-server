select jsonb_path_query('[]', 'strict $.decimal()', silent => true);
