SELECT JSON_VALUE(jsonb 'null', '$ts' PASSING timestamp '2018-02-21 12:34:56 +10' AS ts RETURNING timestamp);
