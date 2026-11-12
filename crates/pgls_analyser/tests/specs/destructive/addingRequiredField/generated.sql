-- Postgres computes the values of generated and identity columns, and serial columns have a default
alter table test add column total int not null generated always as (price * 2) stored;
alter table test add column doubled int not null generated always as (price * 2) virtual;
alter table test add column id int not null generated always as identity;
alter table test add column seq bigserial not null;
-- Only the column without a value is reported
-- expect_lint/addingRequiredField
alter table test add column a int not null default 0, add column b int not null;
-- A NOT NULL constraint (Postgres 18) in the same statement
-- expect_lint/addingRequiredField
alter table test add column c int, add constraint test_c_not_null not null c;
