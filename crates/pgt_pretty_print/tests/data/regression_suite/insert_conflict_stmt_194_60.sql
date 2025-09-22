create table parted_conflict_test_1 partition of parted_conflict_test (b unique) for values in (1, 2);
