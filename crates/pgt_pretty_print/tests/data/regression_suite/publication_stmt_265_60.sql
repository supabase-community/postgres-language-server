CREATE PUBLICATION testpub_dups FOR TABLE testpub_tbl1, testpub_tbl1 (a) WITH (publish = 'insert');
