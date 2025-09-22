create table convslot_test_parent (id int primary key, val int)
partition by range (id);
