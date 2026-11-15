-- expect_lint/requireFunctionSchema
select count(*) from public.users;
-- expect_lint/requireFunctionSchema
select row_number() over () from public.users;
-- expect_lint/requireFunctionSchema
create table public.t (id uuid default gen_random_uuid());
-- expect_lint/requireFunctionSchema
create index on public.users (lower(email));
-- expect_lint/requireFunctionSchema
call refresh_stats();
-- expect_lint/requireFunctionSchema
select * from generate_series(1, 3);
