create table users (id int8, name text);
select id, name from users;
-- expect_lint/unknownColumn
select email from users;
