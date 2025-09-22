create table trigpart4 partition of trigpart for values from (3000) to (4000) partition by range (a);
