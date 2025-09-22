create table rp_prefix_test3 (a int, b int, c int, d int) partition by range(a, b, c, d);
