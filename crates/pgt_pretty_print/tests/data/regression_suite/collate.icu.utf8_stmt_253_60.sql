SELECT a, split_part(b COLLATE ctest_nondet, U&'\00E4b', 2) FROM test6;
