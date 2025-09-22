create table range_list_parted (
	a	int,
	b	char(2)
) partition by range (a);
