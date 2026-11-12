create table items (price int, doubled int generated always as (price * 2) virtual, total int generated always as (price * 2) stored);
-- A virtual generated column (Postgres 18) is computed on read, so there is no rewrite
alter table items alter column doubled type bigint;
-- expect_lint/changingColumnType
alter table items alter column total type bigint;
