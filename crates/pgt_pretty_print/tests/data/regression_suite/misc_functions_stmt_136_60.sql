SELECT t.relname AS toastrel FROM pg_class c
  LEFT JOIN pg_class t ON c.reltoastrelid = t.oid
  WHERE c.relname = 'test_chunk_id'
