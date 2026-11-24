-- The INTO of a set operation is on its left-most select, and creates the table.
select 1 as a into t union select 2;
select a from t;
-- expect_no_diagnostics
