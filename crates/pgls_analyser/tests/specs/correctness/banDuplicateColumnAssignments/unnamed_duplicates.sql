-- Postgres names both scalar subqueries `?column?`.
-- expect_lint/banDuplicateColumnAssignments
create view constants as select (select 1), (select 2);
-- Both are named `max`.
-- expect_lint/banDuplicateColumnAssignments
create view maxima as select (select max(a) from t2), (select max(x) from t2);
-- Without a good name in the ELSE branch, both are named `case`.
-- expect_lint/banDuplicateColumnAssignments
create view cases as select case when true then 1 end, case when true then 'x'::text else 'y' end from t2;
