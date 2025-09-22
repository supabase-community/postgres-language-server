create table parted_copytest (
	a int,
	b int,
	c text
) partition by list (b);
