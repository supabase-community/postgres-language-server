select jsonb_path_query('null', '$.boolean()', silent => true);
