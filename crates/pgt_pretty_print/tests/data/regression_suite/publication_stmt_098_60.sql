CREATE PUBLICATION testpub_rf_yes FOR TABLE testpub_rf_tbl1 WHERE (a > 1) WITH (publish = 'insert');
