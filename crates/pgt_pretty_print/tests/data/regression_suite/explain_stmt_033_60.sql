create table gen_part_1_2
  partition of gen_part_1 for values from (2) to (3);
