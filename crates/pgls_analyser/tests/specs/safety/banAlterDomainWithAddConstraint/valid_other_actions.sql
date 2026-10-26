-- expect_no_diagnostics
alter domain positive_int validate constraint positive_int_check;
alter domain positive_int set default 1;
