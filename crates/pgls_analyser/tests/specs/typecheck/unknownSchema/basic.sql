create schema app;
create table app.users (id int8);
select * from app.users;
-- expect_lint/unknownSchema
select * from missing_schema.users;
