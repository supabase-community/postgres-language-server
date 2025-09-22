SELECT * FROM JSON_TABLE(
	jsonb '[]', '$' AS a
	COLUMNS (
		b int,
		NESTED PATH '$' AS n_a
		COLUMNS (
			c int
		)
	)
) jt;
