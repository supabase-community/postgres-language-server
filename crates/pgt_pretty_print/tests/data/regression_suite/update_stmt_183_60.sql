CREATE TABLE list_parted (
	a text,
	b int
) PARTITION BY list (a);
