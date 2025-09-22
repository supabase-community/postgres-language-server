SELECT * FROM JSON_TABLE (NULL::jsonb, '$' COLUMNS (v1 timestamp)) AS f (v1, v2);
