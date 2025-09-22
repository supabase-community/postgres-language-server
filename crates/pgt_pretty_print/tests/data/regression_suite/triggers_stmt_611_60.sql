create table trg_clone3 partition of trg_clone for values from (2000) to (3000)
  partition by range (a);
