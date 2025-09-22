create table gen_part (
  key1 integer not null,
  key2 integer not null
) partition by list (key1);
