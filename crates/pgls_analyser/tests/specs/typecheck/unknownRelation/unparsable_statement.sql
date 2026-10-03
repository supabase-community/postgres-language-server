-- A statement that doesn't parse, here Postgres 18 syntax, may have created anything, so nothing
-- is reported after it.
-- expect_no_diagnostics
create table virtual_columns (a int, b int generated always as (a * 2) virtual);
select b from virtual_columns;
