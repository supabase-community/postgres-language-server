create table users (id int8, name text);
create table posts (id int8, user_id int8);
select users.id, name from users join posts on posts.user_id = users.id;
select id from users join posts using (id);
-- expect_lint/ambiguousColumn
select id from users join posts on posts.user_id = users.id;
