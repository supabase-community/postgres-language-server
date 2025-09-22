create table part_abc_3 partition of part_abc for values in (3, 4) partition by range (d);
