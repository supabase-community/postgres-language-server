select jsonb_path_query('"12:34:56 +05:30"', '$.time_tz().type()');
