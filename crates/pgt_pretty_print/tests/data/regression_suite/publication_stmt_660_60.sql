CREATE TABLE testpub_insert_onconfl_part_no_ri PARTITION OF testpub_insert_onconfl_parted FOR VALUES FROM (1) TO (10);
