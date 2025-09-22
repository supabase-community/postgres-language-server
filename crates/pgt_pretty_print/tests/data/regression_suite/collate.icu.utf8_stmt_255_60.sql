SELECT a, split_part(b COLLATE ctest_nondet, U&'\00E4b', -1) FROM test6;
