create table gen_part_1
  partition of gen_part for values in (1)
  partition by range (key2);
