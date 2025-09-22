INSERT INTO test_temp SELECT generate_series(1, 10000) as id, repeat('a', 200), 0;
