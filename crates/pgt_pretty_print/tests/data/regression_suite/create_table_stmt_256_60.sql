create table test_part_coll partition of test_part_coll_posix for values from ('a' collate "C") to ('g');
