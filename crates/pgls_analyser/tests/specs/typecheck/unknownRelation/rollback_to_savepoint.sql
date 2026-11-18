create schema app;
create table items (id int8);
begin;
savepoint s;
set search_path to app;
rollback to savepoint s;
select * from items;
savepoint t;
set search_path to app;
release savepoint t;
-- expect_lint/unknownRelation
select * from items;
commit;
