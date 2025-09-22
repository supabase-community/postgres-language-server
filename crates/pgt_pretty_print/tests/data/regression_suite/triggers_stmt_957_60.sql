create table middle partition of grandparent for values from (1) to (10)
partition by range (id);
