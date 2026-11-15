-- A CTE doesn't hide the target of INSERT, UPDATE, DELETE or MERGE: Postgres looks it up through
-- the search_path.
-- expect_lint/requireRelationSchema
with users as (select 1 as id) update users set id = 2;
-- expect_lint/requireRelationSchema
with users as (select 1 as id) insert into users select u.id from users u;
-- expect_lint/requireRelationSchema
with users as (select 1 as id) delete from users;
-- expect_lint/requireRelationSchema
with users as (select 1 as id) merge into users t using users s on t.id = s.id when matched then delete;
