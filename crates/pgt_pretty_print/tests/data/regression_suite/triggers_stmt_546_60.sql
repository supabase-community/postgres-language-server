create table parted_constr_ancestor (a int, b text)
  partition by range (b);
