SELECT JSON_VALUE(jsonb 'null', '$ts' PASSING time '2018-02-21 12:34:56 +10' AS ts RETURNING time);
