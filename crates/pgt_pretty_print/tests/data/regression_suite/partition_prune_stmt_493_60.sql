create table mc3p (a int, b int, c int) partition by range (a, abs(b), c);
