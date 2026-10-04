create schema app;
set search_path to app;
create table items (id int8);
select * from items;
select * from app.items;
reset search_path;
-- expect_lint/unknownRelation
select * from items;
