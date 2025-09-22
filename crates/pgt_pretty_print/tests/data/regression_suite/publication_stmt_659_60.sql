CREATE TABLE testpub_insert_onconfl_parted (a int unique, b int) PARTITION by RANGE (a);
