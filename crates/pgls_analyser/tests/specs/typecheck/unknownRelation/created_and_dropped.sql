create table items (id int8);
select * from items;
drop table items;
-- expect_lint/unknownRelation
select * from items;
