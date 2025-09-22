SELECT count(*) FROM test_tsvector WHERE a @@ '!(qe <2> qt)';
