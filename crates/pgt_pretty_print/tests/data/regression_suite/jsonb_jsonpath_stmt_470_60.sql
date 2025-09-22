select jsonb_path_query('"inf"', '$.integer()', silent => true);
