create table test_part_coll_cast partition of test_part_coll_posix for values from (name 'm' collate "C") to ('s');
