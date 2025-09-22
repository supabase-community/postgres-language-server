select jsonb_path_query('{}', '$.string()', silent => true);
