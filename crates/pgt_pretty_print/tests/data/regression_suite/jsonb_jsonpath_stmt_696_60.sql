select jsonb_path_query('"12:34:56+3"', '$.datetime().type()');
