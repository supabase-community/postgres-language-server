create table part_1_10 partition of range_list_parted for values from (1) to (10) partition by list (b);
