select jsonb_path_query('{}', '$.decimal()', silent => true);
