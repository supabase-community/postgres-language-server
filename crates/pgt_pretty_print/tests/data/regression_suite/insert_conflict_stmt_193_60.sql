create table parted_conflict_test (a int unique, b char) partition by list (a);
