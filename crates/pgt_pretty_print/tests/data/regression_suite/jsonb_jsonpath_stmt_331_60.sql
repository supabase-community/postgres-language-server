select jsonb_path_query('[]', 'strict $.bigint()', silent => true);
