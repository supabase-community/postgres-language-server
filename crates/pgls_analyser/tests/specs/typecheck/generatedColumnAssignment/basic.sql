create table items (
  id int primary key,
  price int,
  quantity int,
  total int generated always as (price * quantity) stored,
  doubled int generated always as (price * 2) virtual
);
insert into items (id, price, quantity) values (1, 2, 3);
insert into items values (2, 2, 3);
insert into items values (3, 2, 3, default, default);
insert into items (id, total) values (4, default), (5, default);
insert into items (id) values (6);
insert into items (id) select 7;
update items set price = 1, total = default;
update items set (price, total) = (1, default);
insert into items (id) values (1) on conflict (id) do update set price = 1, total = default;
-- expect_lint/generatedColumnAssignment
insert into items (id, total) values (1, 2);
-- expect_lint/generatedColumnAssignment
insert into items values (1, 2, 3, 4);
-- expect_lint/generatedColumnAssignment
insert into items (id, doubled) values (1, default), (2, 3);
-- expect_lint/generatedColumnAssignment
insert into items (id, total) select 1, 2;
-- expect_lint/generatedColumnAssignment
insert into items select 1, 2, 3, 4;
-- expect_lint/generatedColumnAssignment
insert into items (id, total) overriding system value values (1, 2);
-- expect_lint/generatedColumnAssignment
update items set total = 1;
-- expect_lint/generatedColumnAssignment
update items set (price, doubled) = (1, 2);
-- expect_lint/generatedColumnAssignment
update items set (price, total) = (select 1, 2);
-- expect_lint/generatedColumnAssignment
insert into items (id) values (1) on conflict (id) do update set total = 1;
-- expect_lint/generatedColumnAssignment
with changed as (update items set doubled = 1 returning id) select id from changed;
