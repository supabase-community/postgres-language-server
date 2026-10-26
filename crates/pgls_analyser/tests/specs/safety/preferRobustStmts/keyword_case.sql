-- The inserted keywords follow the case of the statement.
-- expect_lint/preferRobustStmts
create table users (id bigint);
-- expect_lint/preferRobustStmts
CREATE TABLE posts (id bigint);
