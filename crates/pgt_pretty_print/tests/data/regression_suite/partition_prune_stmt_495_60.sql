create table mc3p1 partition of mc3p
  for values from (1, 1, 1) to (2, minvalue, minvalue);
