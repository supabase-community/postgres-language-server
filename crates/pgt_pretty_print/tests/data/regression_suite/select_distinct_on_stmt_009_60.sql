INSERT INTO distinct_on_tbl SELECT i%10, i%10, i%10 FROM generate_series(1, 1000) AS i;
