create table fk_part_1 partition of fk_part
      (foreign key (a) references fkpart0.pkey) for values in (1)
