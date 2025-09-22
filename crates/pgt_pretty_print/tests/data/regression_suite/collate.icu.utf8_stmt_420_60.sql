CREATE TABLE test12fk (a int, b text COLLATE case_insensitive REFERENCES test12pk (x) ON UPDATE NO ACTION);
