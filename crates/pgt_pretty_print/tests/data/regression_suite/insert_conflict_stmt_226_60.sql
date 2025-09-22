create table parted_conflict (a int primary key, b text) partition by range (a);
