create table part_column_drop (
  useless_1 int,
  id int,
  useless_2 int,
  d int,
  b int,
  useless_3 int
) partition by range (id);
