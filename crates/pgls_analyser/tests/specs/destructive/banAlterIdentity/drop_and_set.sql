-- expect_lint/banAlterIdentity
alter table users alter column id drop identity if exists;
-- expect_lint/banAlterIdentity
alter table users alter column id set generated always;
-- expect_lint/banAlterIdentity
alter table users alter column id set generated always set start with 100;
