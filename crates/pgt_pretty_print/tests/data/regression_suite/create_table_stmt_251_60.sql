create table parted_collate_must_match (a text collate "C", b text collate "C")
  partition by range (a);
