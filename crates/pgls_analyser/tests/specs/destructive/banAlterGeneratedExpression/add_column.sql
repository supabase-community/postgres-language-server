-- expect_lint/banAlterGeneratedExpression
alter table t add column c int generated always as (id + 1) stored;
