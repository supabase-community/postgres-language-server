alter table convslot_test_parent
  attach partition convslot_test_part for values from (1) to (1000);
