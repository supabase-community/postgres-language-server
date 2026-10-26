-- expect_lint/banDuplicateColumnAssignments
-- expect_lint/banDuplicateColumnAssignments
merge into users using new_users on users.id = new_users.id
when matched then update set name = new_users.name, name = 'x'
when not matched then insert (id, id) values (new_users.id, 1);
