create table rangep_0_to_100 partition of rangep for values from (0) to (100) partition by list (b);
