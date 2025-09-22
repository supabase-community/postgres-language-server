ALTER PUBLICATION testpub5 SET TABLE testpub_rf_tbl5 WHERE (xmlexists('//foo[text() = ''bar'']' PASSING BY VALUE a));
