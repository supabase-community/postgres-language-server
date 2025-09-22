create table idxpart (a int, b int unique) partition by range (a, b);
