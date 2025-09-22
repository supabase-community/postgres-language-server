create table idxpart (a int unique, b int) partition by range ((b + a));
