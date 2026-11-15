-- expect_lint/requireColumnQualifier
select u.id from public.users u where name = 'alice';
-- expect_lint/requireColumnQualifier
update public.users u set name = 'bob' where id = 1;
-- expect_lint/requireColumnQualifier
delete from public.posts p where p.id = 1 returning title;
-- A subquery can refer to a column of the query around it.
-- expect_lint/requireColumnQualifier
select u.id from public.users u where u.id in (select p.user_id from public.posts p where p.id = user_id);
-- expect_lint/requireColumnQualifier
insert into public.users (id, name) values (1, 'a') on conflict (id) do update set name = excluded.name where active;
