-- The fix keeps a line comment in the type on a line of its own, so it doesn't comment out the
-- constraints after the type.
-- expect_lint/preferTextField
create table t (name varchar -- length
(10) not null
);
