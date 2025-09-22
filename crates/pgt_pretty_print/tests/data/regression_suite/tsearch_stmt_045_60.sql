SELECT count(*) FROM test_tsvector WHERE a @@ any ('{wr,qh}');
