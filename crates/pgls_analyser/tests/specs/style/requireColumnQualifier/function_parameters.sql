-- The parameters of a SQL function are not columns.
-- expect_lint/requireColumnQualifier
create function public.user_name(user_id integer) returns pg_catalog.text
language sql security definer set search_path = ''
as $$ select u.name from public.users u where u.id = user_id and active $$;
