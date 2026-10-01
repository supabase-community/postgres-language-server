create table typecheck_assignment (qty integer);
-- expect_no_diagnostics
insert into typecheck_assignment values (1);
