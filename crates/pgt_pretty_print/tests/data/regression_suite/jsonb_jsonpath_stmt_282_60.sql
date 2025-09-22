select jsonb_path_query('{}', '$.abs()', silent => true);
