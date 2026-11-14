-- Postgres looks serial up through the search_path outside the columns of CREATE TABLE and
-- ALTER TABLE ... ADD COLUMN, e.g. a domain public.serial.
-- expect_lint/requireTypeSchema
select 1::serial;
-- expect_lint/requireTypeSchema
select 1::"serial";
-- expect_lint/requireTypeSchema
-- expect_lint/requireTypeSchema
create function public.f(a serial) returns bigserial language sql as 'select 1';
-- expect_lint/requireTypeSchema
alter table public.t alter column a type serial;
-- expect_lint/requireTypeSchema
create type public.pair as (a serial);
