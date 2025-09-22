CREATE OPERATOR CLASS part_test_int4_ops_bad FOR TYPE int4 USING hash AS
  FUNCTION 2 part_hashint4_error(int4, int8);
