SELECT relfilenode = 'main_filenode' AS main_same FROM pg_class
  WHERE relname = 'regress_tblspace_test_tbl_idx';
