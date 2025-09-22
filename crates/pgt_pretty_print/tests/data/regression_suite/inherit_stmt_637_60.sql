create table part_10_20 partition of range_list_parted for values from (10) to (20) partition by list (b);
