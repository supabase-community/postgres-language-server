create table idxpart2 partition of idxpart for values from (10, 10) to (20, 20)
  partition by range (b);
