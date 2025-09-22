SELECT JSON_QUERY(jsonb 'null', '$ts' PASSING timestamptz '2018-02-21 12:34:56 +10' AS ts RETURNING jsonb);
