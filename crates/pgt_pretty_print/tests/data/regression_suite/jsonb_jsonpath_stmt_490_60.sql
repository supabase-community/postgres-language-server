select jsonb_path_query('{}', '$.number()', silent => true);
