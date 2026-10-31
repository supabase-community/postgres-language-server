-- Postgres: column reference "id" is ambiguous (42702)
-- expect_lint/ambiguousColumn
-- expect_lint/ambiguousColumn
create table public.mc_a (id int, name text);
create table public.mc_b (id int, label text);
create table public.mc_c (id int, total int);
select id from public.mc_a natural join public.mc_b, public.mc_c;
select id from public.mc_a join public.mc_b using (id) join public.mc_c on true;
-- Only the merged column has the name, or the reference is qualified.
select id, name, label from public.mc_a natural join public.mc_b;
select mc_c.id, total from public.mc_a natural join public.mc_b, public.mc_c;
