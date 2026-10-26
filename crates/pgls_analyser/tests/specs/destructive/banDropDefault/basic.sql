-- expect_lint/banDropDefault
alter table users alter column created_at drop default;
-- Each dropped default is reported.
-- expect_lint/banDropDefault
-- expect_lint/banDropDefault
alter table users alter column created_at drop default, alter column updated_at set default now(), alter updated_by drop default;
