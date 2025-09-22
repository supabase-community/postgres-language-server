select jsonb_path_query_tz('"2023-08-15 12:34:56 +5:30"', '$.timestamp().string()');
