create table partitioned2
  partition of partitioned for values in ('(2,4)'::partitioned);
