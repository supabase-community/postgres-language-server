select jsonb_path_query('"12:34:56 +5:30"', '$.time_tz().string()');
