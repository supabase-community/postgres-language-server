select jsonb_path_query('null', '$.integer()', silent => true);
