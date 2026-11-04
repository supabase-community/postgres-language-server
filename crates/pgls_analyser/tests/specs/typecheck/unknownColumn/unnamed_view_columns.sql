-- Postgres names these view columns `max`, `arr` and `a`, so the selects are accepted.
-- expect_no_diagnostics
create table public.t2 (a int);
create view public.va as select (select max(a) from public.t2);
select max from public.va;
create type public.comp2 as (arr int[]);
create table public.t3 (c public.comp2);
create view public.vc as select (t3.c).arr[1] from public.t3;
select arr from public.vc;
create view public.vd as select case when true then 1 else a end from public.t2;
select a from public.vd;
