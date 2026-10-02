create table users (id int8);
select u.id from users u;
-- expect_lint/missingFromClauseEntry
select users.id from users u;
