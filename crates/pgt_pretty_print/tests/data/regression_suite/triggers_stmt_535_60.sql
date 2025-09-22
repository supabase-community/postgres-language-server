create table parted (a int, b int, c text) partition by list ((a + b));
