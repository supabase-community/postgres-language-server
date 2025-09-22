create table mlparted5_cd partition of mlparted5
  for values from ('c') to ('e') partition by list (c);
