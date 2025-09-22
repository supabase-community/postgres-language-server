select jsonb_path_query('null', '$.number()', silent => true);
