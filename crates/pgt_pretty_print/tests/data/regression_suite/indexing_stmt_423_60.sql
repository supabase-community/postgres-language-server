create table idxpart (a int, b text, primary key (a, b)) partition by range (a);
