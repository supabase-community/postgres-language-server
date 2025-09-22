create table parted_index_col_drop(a int, b int, c int)
  partition by list (a);
