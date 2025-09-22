create table partitioned (
	a intdom1,
	b text
) partition by range (plusone(a));
