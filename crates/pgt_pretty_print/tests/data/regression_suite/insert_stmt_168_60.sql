create table part_gg2 partition of part_gg for values from (1) to (10) partition by range (b);
