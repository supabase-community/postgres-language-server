select jsonb_path_query('"12:34:56"', '$.datetime("HH24:MI:SS").type()');
