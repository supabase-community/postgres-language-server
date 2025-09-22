create table idxpart1 partition of idxpart for values from (1) to (1000) partition by range (a);
