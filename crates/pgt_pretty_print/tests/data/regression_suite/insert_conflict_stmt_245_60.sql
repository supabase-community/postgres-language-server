alter table parted_conflict attach partition parted_conflict_1 for values from (0) to (1000);
