-- expect_lint/preferTextField
-- expect_lint/preferTextField
create table users (
    id bigint primary key,
    name varchar(255) not null,
    email character varying (100)[],
    bio varchar
);
