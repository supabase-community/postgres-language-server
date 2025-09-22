create table hp (a int, b text, c int)
  partition by hash (a part_test_int4_ops, b part_test_text_ops);
