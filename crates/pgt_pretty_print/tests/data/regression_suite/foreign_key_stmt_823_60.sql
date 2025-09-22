create table fk_part (a int, constraint fkey foreign key (a) references fkpart2.pkey) partition by list (a)
