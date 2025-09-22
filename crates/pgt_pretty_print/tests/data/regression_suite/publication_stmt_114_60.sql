CREATE PUBLICATION testpub_dups FOR TABLE testpub_rf_tbl1, testpub_rf_tbl1 WHERE (a = 2) WITH (publish = 'insert');
