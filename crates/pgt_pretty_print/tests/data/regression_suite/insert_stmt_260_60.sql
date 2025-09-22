create table mlparted5 partition of mlparted
  for values from (1, 40) to (1, 50) partition by range (c);
