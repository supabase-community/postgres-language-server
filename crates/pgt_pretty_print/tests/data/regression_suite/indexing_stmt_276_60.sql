create table idxpart(drop_1 int, drop_2 int, col_keep int, drop_3 int) partition by range (col_keep);
