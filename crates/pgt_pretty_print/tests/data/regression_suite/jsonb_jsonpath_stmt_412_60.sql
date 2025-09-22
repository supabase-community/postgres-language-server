select jsonb_path_query('null', '$.decimal()', silent => true);
