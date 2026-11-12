-- Qualified columns, and the references that are not reported.
-- expect_no_diagnostics
select at.relid from pg_catalog.pg_statio_all_tables as at;
select 1, pg_catalog.now();
select at.relid as id, at.relname from pg_catalog.pg_statio_all_tables at order by id, relname, 1;
select at.relname from pg_catalog.pg_statio_all_tables at group by relname;
select u.id from public.users u union select p.id from public.posts p order by id;
select u from public.users u;
insert into public.users (id, name) values (1, 'a') on conflict (id) do nothing;
update public.users u set name = 'bob' where u.id = 1;
select * from public.users join public.posts using (id);
create table public.t (a integer check (a > 0), b integer default 1);
create index on public.t (b) where a > 0;
