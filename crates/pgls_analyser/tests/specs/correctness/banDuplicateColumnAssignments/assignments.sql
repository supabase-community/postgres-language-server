-- expect_lint/banDuplicateColumnAssignments
update users set (name, email) = ('a', 'b'), name = 'c';
-- expect_lint/banDuplicateColumnAssignments
insert into users (id, name, id) values (1, 'a', 2);
-- expect_lint/banDuplicateColumnAssignments
insert into users (id, name) values (1, 'a') on conflict (id) do update set name = 'b', name = excluded.name;
-- expect_lint/banDuplicateColumnAssignments
with moved as (update users set name = 'a', name = 'b' returning id) select * from moved;
-- expect_lint/banDuplicateColumnAssignments
update users set tags[1] = 'a', tags = '{}';
