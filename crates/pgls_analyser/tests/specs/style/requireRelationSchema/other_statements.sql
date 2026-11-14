-- expect_lint/requireRelationSchema
lock table users;
-- expect_lint/requireRelationSchema
prepare p as select * from users;
-- expect_lint/requireRelationSchema
copy users to stdout;
-- expect_lint/requireRelationSchema
explain select * from users;
-- expect_lint/requireRelationSchema
declare c cursor for select * from users;
