select jsonb_path_query('{}', '$.boolean()', silent => true);
