CREATE TABLE addr_nsp.parttable (
    a int PRIMARY KEY
) PARTITION BY RANGE (a);
