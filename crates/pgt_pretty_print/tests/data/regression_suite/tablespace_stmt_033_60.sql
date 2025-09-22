SELECT relfilenode as toast_filenode FROM pg_class
  WHERE oid =
    (SELECT i.indexrelid
       FROM pg_class c,
            pg_index i
       WHERE i.indrelid = c.reltoastrelid AND
             c.relname = 'regress_tblspace_test_tbl') ;
