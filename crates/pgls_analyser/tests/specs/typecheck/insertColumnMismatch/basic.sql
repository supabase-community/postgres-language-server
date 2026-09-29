create table users (id int8, name text);
insert into users (id, name) values (1, 'a');
insert into users values (1);
insert into users (id) select id from users;
-- expect_lint/insertColumnMismatch
insert into users (id, name) values (1);
-- expect_lint/insertColumnMismatch
insert into users values (1, 'a', 'b');
