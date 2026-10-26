-- expect_lint/banDuplicateColumnAssignments
update users set name = 'a', name = 'b';
