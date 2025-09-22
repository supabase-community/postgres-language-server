create table mcrparted (a int, b int, c int) partition by range (a, abs(b), c);
