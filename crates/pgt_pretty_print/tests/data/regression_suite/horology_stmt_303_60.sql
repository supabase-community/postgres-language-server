SELECT i, to_timestamp('2018-11-02 12:34:56', 'YYYY-MM-DD HH24:MI:SS.FF' || i) FROM generate_series(1, 6) i;
