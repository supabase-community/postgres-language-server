SELECT a, string_to_array(b COLLATE ctest_nondet, U&'\00E4b') FROM test6;
