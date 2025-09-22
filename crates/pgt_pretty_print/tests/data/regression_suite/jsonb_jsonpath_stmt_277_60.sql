select jsonb_path_query('"inf"', '$.double()', silent => true);
