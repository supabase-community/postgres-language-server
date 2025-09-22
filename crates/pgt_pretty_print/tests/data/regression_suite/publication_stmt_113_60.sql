CREATE PUBLICATION testpub_dups FOR TABLE testpub_rf_tbl1 WHERE (a = 1), testpub_rf_tbl1 WITH (publish = 'insert');
