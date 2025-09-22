ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl1 WHERE (COALESCE(b, 'foo') = 'foo');
