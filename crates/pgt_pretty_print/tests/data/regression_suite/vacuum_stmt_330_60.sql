SELECT pg_column_toast_chunk_id(f1) = 'id_2_chunk' AS same_chunk
  FROM vac_rewrite_toast WHERE id = 2;
