select jsonb_path_query('{}', '$.integer()', silent => true);
