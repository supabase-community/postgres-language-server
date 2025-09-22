CREATE PUBLICATION testpub_col_list FOR TABLE testpub_tbl8 (a, b) WITH (publish_via_partition_root = 'true');
