-- expect_no_diagnostics
create table public.posts (id bigint);
alter table public.posts add column title text;
create table public.post_ids as select id from public.posts;
drop table public.posts;
create temp table scratch (id bigint);
create temporary table scratch_ids as select 1 as id;
create view post_titles as select title from public.posts;
