create table items (
  price int,
  total int generated always as (price * 2) stored
);
-- `LIKE` copies a generated column as an ordinary column, unless it includes generated columns.
create table plain_items (like items);
insert into plain_items (price, total) values (1, 2);
-- expect_lint/generatedColumnAssignment
create table generated_items (like items including generated);
insert into generated_items (price, total) values (1, 2);
-- expect_lint/generatedColumnAssignment
create table child_items (extra int) inherits (items);
insert into child_items (price, total) values (1, 2);
create table partitioned_items (
  price int,
  total int generated always as (price * 2) stored
) partition by range (price);
-- expect_lint/generatedColumnAssignment
create table partition_items partition of partitioned_items for values from (0) to (10);
insert into partition_items (price, total) values (1, 2);
-- expect_lint/generatedColumnAssignment
alter table plain_items add column doubled int generated always as (price * 2) virtual;
update plain_items set doubled = 1;
-- A view's columns are not generated: the error depends on how the view is updated.
create view item_view as select price, total from items;
insert into item_view (price, total) values (1, 2);
-- `DROP EXPRESSION` turns the column into an ordinary column, in the children too.
alter table items alter column total drop expression;
insert into items (price, total) values (1, 2);
insert into child_items (price, total) values (1, 2);
-- `SELECT *` assigns the columns of its FROM items.
create table generated_dst (a int, g int generated always as (a * 2) stored);
create table plain_src (a int, g int);
-- expect_lint/generatedColumnAssignment
insert into generated_dst select * from plain_src;
-- expect_lint/generatedColumnAssignment
insert into generated_dst select plain_src.* from plain_src;
insert into generated_dst select a from plain_src;
