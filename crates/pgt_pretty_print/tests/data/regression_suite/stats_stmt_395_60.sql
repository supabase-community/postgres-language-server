INSERT INTO test_io_local SELECT generate_series(1, 5000) as id, repeat('a', 200);
