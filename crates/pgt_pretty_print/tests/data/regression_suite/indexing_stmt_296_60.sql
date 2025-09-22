create table idxpart (a int, b int, c text, primary key  (a, b, c)) partition by range (b, c, a);
