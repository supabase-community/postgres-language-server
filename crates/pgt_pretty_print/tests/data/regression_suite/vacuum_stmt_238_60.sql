SELECT t.relfilenode = 'toast_filenode' AS is_same_toast_filenode
  FROM pg_class c, pg_class t
  WHERE c.reltoastrelid = t.oid AND c.relname = 'vac_option_tab';
