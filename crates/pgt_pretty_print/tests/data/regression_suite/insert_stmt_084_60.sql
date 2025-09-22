create domain insert_test_domain as insert_test_type
  check ((value).if1 is not null and (value).if2 is not null);
