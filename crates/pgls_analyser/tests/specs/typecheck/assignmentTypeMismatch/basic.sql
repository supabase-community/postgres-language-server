create table typecheck_assignment (qty integer);
-- expect_lint/assignmentTypeMismatch
insert into typecheck_assignment values (timestamp '2020-01-01');
