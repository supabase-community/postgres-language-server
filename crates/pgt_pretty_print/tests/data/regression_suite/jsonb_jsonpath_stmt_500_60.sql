select jsonb_path_query('"-inf"', '$.number()', silent => true);
