-- expect_lint/requireRelationSchema
insert into users (name) values ('alice');
-- expect_lint/requireRelationSchema
update users set name = 'bob' where public.users.id = 1;
-- expect_lint/requireRelationSchema
-- expect_lint/requireRelationSchema
delete from posts using users where posts.user_id = users.id;
-- expect_lint/requireRelationSchema
-- expect_lint/requireRelationSchema
merge into users u using staged s on u.id = s.id when matched then update set name = s.name;
