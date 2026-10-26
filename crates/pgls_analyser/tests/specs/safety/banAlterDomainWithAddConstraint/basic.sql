-- expect_lint/banAlterDomainWithAddConstraint
alter domain positive_int add constraint positive_int_check check (value > 0);
