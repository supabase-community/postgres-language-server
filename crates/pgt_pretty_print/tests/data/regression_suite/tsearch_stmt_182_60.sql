SELECT count(*) FROM test_tsvector WHERE a @@ 'wr' AND a @@ '!qh';
