select jsonb_path_query('true', '$.decimal()', silent => true);
