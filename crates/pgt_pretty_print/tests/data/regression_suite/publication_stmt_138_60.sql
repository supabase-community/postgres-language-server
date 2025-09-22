CREATE PUBLICATION testpub6 FOR TABLE testpub_rf_tbl1 WHERE (a IN (SELECT generate_series(1,5)));
