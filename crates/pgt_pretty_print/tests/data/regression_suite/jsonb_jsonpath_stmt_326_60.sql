select jsonb_path_query('null', '$.bigint()', silent => true);
