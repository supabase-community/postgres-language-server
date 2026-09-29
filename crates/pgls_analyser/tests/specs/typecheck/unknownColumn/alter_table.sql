create table users (id int8, name text);
alter table users add column email text;
select email from users;
alter table users drop column name;
alter table users rename column email to address;
select address from users;
-- expect_lint/unknownColumn
select name from users;
-- expect_lint/unknownColumn
update users set email = 'x';
