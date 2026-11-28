-- expect_lint/banAlterGeneratedExpression
alter table t alter column c set expression as (id + 2);
