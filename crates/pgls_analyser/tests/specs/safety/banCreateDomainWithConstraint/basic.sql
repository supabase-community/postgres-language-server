-- expect_lint/banCreateDomainWithConstraint
create domain positive_int as int check (value > 0);
