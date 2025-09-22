CREATE TABLE sub_parted PARTITION OF list_parted for VALUES in (1) PARTITION BY list (b);
