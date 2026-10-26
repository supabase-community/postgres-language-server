-- expect_lint/banDropDefault
alter domain status drop default;
-- expect_lint/banDropDefault
alter view active_users alter column created_at drop default;
