-- The column list of a CTE renames its typed columns, so `x` is an integer.
-- expect_lint/operatorTypeMismatch
with c(x) as (select 1 as a) select x + timestamp '2020-01-01' from c;
