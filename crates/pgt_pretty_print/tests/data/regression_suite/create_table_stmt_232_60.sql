CREATE TABLE part_b PARTITION OF parted (
	b NOT NULL DEFAULT 1,
	CONSTRAINT check_a CHECK (length(a) > 0),
	CONSTRAINT check_b CHECK (b >= 0)
) FOR VALUES IN ('b');
