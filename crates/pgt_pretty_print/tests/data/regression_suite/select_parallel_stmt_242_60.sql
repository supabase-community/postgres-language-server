INSERT INTO parallel_hang
	(SELECT * FROM generate_series(1, 400) gs);
