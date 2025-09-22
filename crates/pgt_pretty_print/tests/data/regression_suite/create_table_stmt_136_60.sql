CREATE TABLE part_bogus_expr_fail PARTITION OF list_parted FOR VALUES IN ((1+1) collate "POSIX");
