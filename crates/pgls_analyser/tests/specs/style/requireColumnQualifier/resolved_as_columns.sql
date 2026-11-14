-- Names of relations, aliases, parameters and output columns that Postgres resolves to a column.
-- catalog: builtin
create table public.accounts (id integer, secret text);
create table public.posts (id integer);
create table public.collision (u integer);
create table public.users (id integer, name text);
create table public.group_probe (id integer, n integer);
-- The alias secret of the subquery is not visible from the outer query.
-- expect_lint/requireColumnQualifier
select secret from public.accounts a where exists (select 1 from public.posts secret);
-- The column u takes precedence over the whole row of u.
-- expect_lint/requireColumnQualifier
select u from public.collision u;
-- The column id takes precedence over the parameter id.
-- expect_lint/requireColumnQualifier
create function public.user_id(id integer) returns integer language sql
as $$ select id from public.users u $$;
-- In GROUP BY, the column n takes precedence over the output column n, also in grouping sets.
-- expect_lint/requireColumnQualifier
select u.id as n, pg_catalog.count(*) from public.group_probe u group by n, u.id;
-- expect_lint/requireColumnQualifier
select u.id as n from public.group_probe u group by rollup (n);
