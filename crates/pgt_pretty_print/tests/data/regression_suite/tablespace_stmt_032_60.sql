SELECT relfilenode as main_filenode FROM pg_class
  WHERE relname = 'regress_tblspace_test_tbl_idx' ;
