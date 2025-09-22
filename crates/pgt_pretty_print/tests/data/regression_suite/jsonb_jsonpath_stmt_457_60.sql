select jsonb_path_query('true', '$.integer()', silent => true);
