select jsonb_path_query('"2023-08-15 12:34:56.789 +05:30"', '$.timestamp_tz(2.0)');
