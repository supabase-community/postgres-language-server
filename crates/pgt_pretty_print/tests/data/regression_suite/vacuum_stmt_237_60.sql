SELECT relfilenode = 'main_filenode' AS is_same_main_filenode
  FROM pg_class WHERE relname = 'vac_option_tab';
