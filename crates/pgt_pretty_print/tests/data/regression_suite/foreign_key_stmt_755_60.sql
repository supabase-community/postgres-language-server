alter table other_partitioned_fk add foreign key (a, b)
  references fk_notpartitioned_pk(a, b);
