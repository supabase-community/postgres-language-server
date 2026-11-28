-- expect_lint/banAlterRowLevelSecurity
alter table t disable row level security;
-- expect_lint/banAlterRowLevelSecurity
alter table t force row level security;
-- expect_lint/banAlterRowLevelSecurity
alter table t no force row level security;
