-- expect_lint/banAlterRoleOptions
alter user app nobypassrls;
-- expect_lint/banAlterRoleOptions
alter user app in database db set search_path to private;
-- expect_lint/banAlterRoleOptions
alter user app reset all;
