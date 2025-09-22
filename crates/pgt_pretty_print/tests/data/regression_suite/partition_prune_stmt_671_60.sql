create table rp_prefix_test2 (a int, b int, c int) partition by range(a, b, c);
