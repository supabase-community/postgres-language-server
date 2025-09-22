select jsonb_path_query_tz('"12:34:56"', '$.time_tz().string()');
