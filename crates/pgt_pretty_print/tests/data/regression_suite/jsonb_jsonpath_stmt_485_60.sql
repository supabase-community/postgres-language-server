select jsonb_path_query('true', '$.number()', silent => true);
