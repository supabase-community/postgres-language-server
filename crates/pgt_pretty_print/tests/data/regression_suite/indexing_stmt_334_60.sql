create table idxpart (a int4range, b int4range) partition by range (a, b);
