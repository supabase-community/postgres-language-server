create table hash_parted (
	a int
) partition by hash (a part_test_int4_ops);
