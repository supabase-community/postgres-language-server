SELECT t.relfilenode AS toast_filenode FROM pg_class c, pg_class t
  WHERE c.reltoastrelid = t.oid AND c.relname = 'vac_option_tab' ;
