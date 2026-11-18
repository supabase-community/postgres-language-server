-- The INTO of a set operation is on its left-most select.
-- expect_lint/requireTableSchema
select 1 as a into t union select 2;
