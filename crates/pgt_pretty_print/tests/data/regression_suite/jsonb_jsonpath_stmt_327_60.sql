select jsonb_path_query('true', '$.bigint()', silent => true);
