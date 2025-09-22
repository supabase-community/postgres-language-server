SELECT count(*) FROM test_tsvector WHERE a @@ '!no_such_lexeme';
