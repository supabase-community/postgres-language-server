-- expect_lint/requireTableSchema
alter table posts add column title text;
-- expect_lint/requireTableSchema
create table post_ids as select id from public.posts;
-- expect_lint/requireTableSchema
drop table public.comments, posts;
