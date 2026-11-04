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
-- A column that redeclares an inherited ordinary column can make it generated (Postgres 15).
create table plain_parent (a int, g int);
create table redeclared_child (g int generated always as (a * 2) stored) inherits (plain_parent);
-- expect_lint/generatedColumnAssignment
insert into redeclared_child values (1, 2);
-- An attached partition follows the `DROP EXPRESSION` of its partitioned table.
create table attach_parent (
  a int,
  g int generated always as (a * 2) stored
) partition by range (a);
create table attached (like attach_parent including generated);
alter table attach_parent attach partition attached for values from (0) to (10);
alter table attach_parent alter column g drop expression;
insert into attached values (1, 2);
-- A detached partition doesn't.
create table detach_parent (
  a int,
  g int generated always as (a * 2) stored
) partition by range (a);
create table detached partition of detach_parent for values from (0) to (10);
alter table detach_parent detach partition detached;
alter table detach_parent alter column g drop expression;
-- expect_lint/generatedColumnAssignment
insert into detached values (1, 2);
-- The same for `INHERIT` and `NO INHERIT`.
create table inherit_parent (a int, g int generated always as (a * 2) stored);
create table inherit_child (a int, g int generated always as (a * 2) stored);
alter table inherit_child inherit inherit_parent;
create table uninherited_child () inherits (inherit_parent);
alter table uninherited_child no inherit inherit_parent;
alter table inherit_parent alter column g drop expression;
insert into inherit_child values (1, 2);
-- expect_lint/generatedColumnAssignment
insert into uninherited_child values (1, 2);
