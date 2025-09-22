select jsonb_path_query('"12:34:56 +05:30"', '$.timestamp_tz()');
