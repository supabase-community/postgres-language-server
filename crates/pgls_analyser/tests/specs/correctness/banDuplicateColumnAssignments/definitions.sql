-- expect_lint/banDuplicateColumnAssignments
create table users (id bigint, name text, id text);
-- expect_lint/banDuplicateColumnAssignments
create view user_ids (id, id) as select 1, 2;
-- expect_lint/banDuplicateColumnAssignments
create view user_ids as select 1 as id, 2 as id;
-- expect_lint/banDuplicateColumnAssignments
create view user_pairs as select a.id, b.id from users a, users b;
-- expect_lint/banDuplicateColumnAssignments
create view user_counts as select count(*), count(*) from users;
-- expect_lint/banDuplicateColumnAssignments
create table user_pairs as select a.id, b.id from users a, users b;
-- expect_lint/banDuplicateColumnAssignments
create materialized view user_names (name) as select 'x', 1 as name;
-- expect_lint/banDuplicateColumnAssignments
select 1 as id, 2 as id into user_ids;
-- expect_lint/banDuplicateColumnAssignments
create view user_ids as select 1 as id, 2 as id union select 3, 4;
