create table hash_parted (
	a int,
	b int
) partition by hash (a custom_opclass, b custom_opclass);
