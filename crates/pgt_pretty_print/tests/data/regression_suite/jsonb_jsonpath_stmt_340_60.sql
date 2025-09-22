select jsonb_path_query('"inf"', '$.bigint()', silent => true);
