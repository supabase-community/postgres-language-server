create table parted_conflict_test_4 partition of parted_conflict_test for values in (5) partition by list (a);
