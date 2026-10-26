-- expect_no_diagnostics
begin;
create table users (id bigint);
commit;
begin;
create table posts (id bigint);
rollback;
