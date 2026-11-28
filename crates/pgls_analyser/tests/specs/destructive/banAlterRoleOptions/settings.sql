-- expect_lint/banAlterRoleOptions
alter role r set search_path to public;
-- expect_lint/banAlterRoleOptions
alter role r reset search_path;
