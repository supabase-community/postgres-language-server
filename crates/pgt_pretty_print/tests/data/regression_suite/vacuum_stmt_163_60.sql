CREATE TABLE vacparted_i (a int primary key, b varchar(100))
  PARTITION BY HASH (a);
