create table mlparted5_ab partition of mlparted5
  for values from ('a') to ('c') partition by list (c);
