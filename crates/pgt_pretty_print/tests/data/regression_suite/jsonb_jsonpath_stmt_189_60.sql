select jsonb_path_query('1', '$ + "2"', silent => true);
