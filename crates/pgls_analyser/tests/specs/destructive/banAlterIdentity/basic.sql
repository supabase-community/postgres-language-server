-- expect_lint/banAlterIdentity
alter table users alter column id add generated always as identity;
