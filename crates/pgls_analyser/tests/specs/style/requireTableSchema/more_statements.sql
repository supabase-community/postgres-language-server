-- expect_lint/requireTableSchema
create foreign table remote_users (id int) server remote;
-- expect_lint/requireTableSchema
alter table users rename to customers;
-- expect_lint/requireTableSchema
alter table users rename column name to full_name;
-- expect_lint/requireTableSchema
alter table users set schema app;
-- expect_lint/requireTableSchema
select 1 as id into user_ids;
-- expect_lint/requireTableSchema
create index on users (id);
-- expect_lint/requireTableSchema
create table public.posts (user_id int references users (id));
-- expect_lint/requireTableSchema
create table public.users_2024 partition of users for values in (2024);
-- expect_lint/requireTableSchema
alter table public.users attach partition users_2025 for values in (2025);
-- expect_lint/requireTableSchema
alter table public.posts add constraint posts_user_fk foreign key (user_id) references users (id);
-- expect_lint/requireTableSchema
create table public.archived_users (like users);
-- expect_lint/requireTableSchema
drop foreign table remote_users;
