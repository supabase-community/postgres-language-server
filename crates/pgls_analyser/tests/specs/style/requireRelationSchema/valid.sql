-- Qualified relations, CTEs and temporary tables are not reported, nor the tables of DDL,
-- which requireTableSchema checks.
-- expect_no_diagnostics
select * from pg_catalog.pg_statio_all_tables;
with recent as (select * from public.posts) select * from recent;
with recursive t as (select 1 as n union all select n + 1 from t where n < 3) select * from t;
create temp table scratch (id int);
insert into scratch select id from public.users;
select * from scratch;
create table posts (id bigint);
select 1 as id into user_ids;
select * from public.users for update of users;
