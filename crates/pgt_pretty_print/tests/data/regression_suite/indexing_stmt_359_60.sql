create table idxpart1 (a int not null, b int, unique (a, b))
  partition by range (a, b);
