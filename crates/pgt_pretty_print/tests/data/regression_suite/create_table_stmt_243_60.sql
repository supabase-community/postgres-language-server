create table parted_notnull_inh_test (a int default 1, b int not null default 0) partition by list (a);
