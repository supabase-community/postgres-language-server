-- expect_lint/operatorTypeMismatch
select 1 + timestamp '2020-01-01';
-- expect_lint/operatorTypeMismatch
select - true;
