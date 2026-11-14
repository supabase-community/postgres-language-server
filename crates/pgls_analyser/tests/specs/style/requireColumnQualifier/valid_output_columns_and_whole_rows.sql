-- Output columns, whole rows and parameters that no column hides.
-- catalog: builtin
-- expect_no_diagnostics
create table public.users (id integer, name text);
select u.name::pg_catalog.text from public.users u order by name;
select u.name as id from public.users u order by id;
select u.id as n from public.users u group by grouping sets ((n));
select u.id as n from public.users u group by rollup (n), cube ((n, u.name));
select u.id as n, u.name as m from public.users u group by (n, m);
select u from public.users u;
create function public.user_name(user_id integer) returns pg_catalog.text language sql
as $$ select u.name from public.users u where u.id = user_id $$;
