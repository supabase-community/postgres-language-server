create domain insert_test_domain as insert_test_type
  check ((value).if2[1] is not null);
