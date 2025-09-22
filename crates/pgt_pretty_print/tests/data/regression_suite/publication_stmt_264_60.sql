CREATE PUBLICATION testpub_dups FOR TABLE testpub_tbl1 (a), testpub_tbl1 WITH (publish = 'insert');
