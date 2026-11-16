-- Postgres names `(select * from one)` after the first column of `one`, which the catalog knows.
create table one (a int);
-- expect_lint/banDuplicateColumnAssignments
create view first_column as select (select * from one), 1 as a;
-- expect_lint/banDuplicateColumnAssignments
create table first_column_table as select 1 as a, (select * from one limit 1);
-- expect_lint/banDuplicateColumnAssignments
select (select a from one), (select one.* from one) into first_column_into;
-- Other names, and a table whose columns are unknown.
create view other_column as select (select * from one), 1 as b;
create view unknown_table as select (select * from nope), 1 as a;
