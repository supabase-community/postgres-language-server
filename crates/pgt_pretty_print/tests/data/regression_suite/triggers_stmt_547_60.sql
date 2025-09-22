create table parted_constr (a int, b text)
  partition by range (b);
