create table partitioned (a int, b int)
  partition by list ((row(a, b)::partitioned));
