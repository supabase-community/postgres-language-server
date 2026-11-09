-- Postgres: operator does not exist: text - integer
-- expect_lint/operatorTypeMismatch
create table public.nj_a (id int, label text);
create table public.nj_b (id int8, amount int);
select label - amount from public.nj_a natural join public.nj_b;
-- The merged column has the common type of both sides.
select id + 1, amount + 1 from public.nj_a natural join public.nj_b;
