create table parent (a int primary key, f int references parent)
  partition by list (a);
