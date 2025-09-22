INSERT INTO tbl SELECT x, 2*x, NULL, NULL FROM generate_series(1,300) AS x;
