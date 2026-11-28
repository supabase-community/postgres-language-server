-- expect_lint/banAlterFunctionOptions
alter procedure p() security definer;
-- expect_lint/banAlterFunctionOptions
alter procedure p() set search_path to private;
-- expect_lint/banAlterFunctionOptions
alter procedure p() reset all;
-- expect_lint/banAlterFunctionOptions
alter routine f() security invoker;
-- expect_lint/banAlterFunctionOptions
alter routine f() set search_path to private;
-- expect_lint/banAlterFunctionOptions
alter routine f() reset all;
