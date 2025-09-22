select jsonb_path_query('"12:34 +05"', '$.datetime("HH24:MI TZH")');
