-- expect_lint/banDropConstraint
alter table users drop constraint users_email_key;
-- Each dropped constraint is reported.
-- expect_lint/banDropConstraint
-- expect_lint/banDropConstraint
alter table users drop constraint users_email_key, add column age int, drop constraint if exists users_name_check;
