create table targets (id int);
-- expect_lint/invalidSelectInto
select 1 as id union select 2 into ids;
-- expect_lint/invalidSelectInto
select 1 as id union (select 2 into ids union select 3);
-- expect_lint/invalidSelectInto
insert into targets select 1 union all select 2 into ids;
