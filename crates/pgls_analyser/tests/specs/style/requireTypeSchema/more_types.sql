-- expect_lint/requireTypeSchema
create table public.users (id uuid, name varchar(100));
-- expect_lint/requireTypeSchema
-- expect_lint/requireTypeSchema
create function public.f(x jsonb) returns text language sql as 'select pg_catalog.jsonb_typeof(x)';
-- expect_lint/requireTypeSchema
select cast('{}' as jsonb);
-- expect_lint/requireTypeSchema
create domain public.email as citext;
