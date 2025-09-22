select jsonb_path_query('"12:34 +05:20"', '$.datetime("HH24:MI TZH:TZM")');
