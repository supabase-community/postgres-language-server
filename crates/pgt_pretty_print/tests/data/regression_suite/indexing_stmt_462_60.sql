create table idxpart_another (a int, b int, primary key (a, b)) partition by range (a);
