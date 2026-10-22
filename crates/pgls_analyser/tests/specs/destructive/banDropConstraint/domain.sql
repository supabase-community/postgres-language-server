-- expect_lint/banDropConstraint
alter domain positive_int drop constraint if exists positive_int_check;
