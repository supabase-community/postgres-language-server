-- The fix keeps the last assignment.
-- expect_lint/banDuplicateColumnAssignments
update users set name = 'a', email = 'b', name = 'c' /* last */, age = 1, name = 'd';
-- expect_lint/banDuplicateColumnAssignments
insert into users (id, name) values (1, 'a') on conflict (id) do update set name = 'b', name = excluded.name;
-- No fix for column lists and multiple-column assignments.
-- expect_lint/banDuplicateColumnAssignments
insert into users (id, name, id) values (1, 'a', 2);
-- expect_lint/banDuplicateColumnAssignments
update users set (name, email) = ('a', 'b'), name = 'c';
-- expect_lint/banDuplicateColumnAssignments
update users set tags[1] = 'a', tags = '{}';
