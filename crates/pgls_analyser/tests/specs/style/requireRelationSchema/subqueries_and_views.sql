-- expect_lint/requireRelationSchema
select * from public.users where id in (select user_id from posts);
-- expect_lint/requireRelationSchema
with recent as (select * from posts) select * from recent;
-- expect_lint/requireRelationSchema
create view public.active_users as select * from users;
-- expect_lint/requireRelationSchema
create materialized view public.post_counts as select count(*) from public.posts join comments on true;
-- expect_lint/requireRelationSchema
truncate posts;
