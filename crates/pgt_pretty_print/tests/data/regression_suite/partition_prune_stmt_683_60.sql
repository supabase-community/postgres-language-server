create table hp_prefix_test (a int, b int, c int, d int)
  partition by hash (a part_test_int4_ops, b part_test_int4_ops, c part_test_int4_ops, d part_test_int4_ops);
