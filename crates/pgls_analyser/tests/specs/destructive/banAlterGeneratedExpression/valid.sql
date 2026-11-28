-- expect_no_diagnostics
alter table t add column c int;
alter table t alter column c drop expression;
alter table t add column id bigint generated always as identity;
