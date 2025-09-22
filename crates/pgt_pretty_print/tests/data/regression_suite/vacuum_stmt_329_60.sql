SELECT id, pg_column_toast_chunk_id(f1) IS NULL AS f1_chunk_null,
  substr(f1, 5, 10) AS f1_data,
  pg_column_compression(f1) AS f1_comp
  FROM vac_rewrite_toast ORDER BY id;
