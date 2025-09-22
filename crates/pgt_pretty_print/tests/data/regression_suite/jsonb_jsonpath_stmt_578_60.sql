select jsonb_path_query('"12:34:56.789 +05:30"', '$.time_tz(-1)');
