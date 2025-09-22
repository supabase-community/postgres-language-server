CREATE PUBLICATION testpub6 FOR TABLE testpub_rf_tbl1 WHERE ('(0,1)'::tid = ctid);
