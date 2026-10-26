-- The fix keeps the comment and the SQL valid.
-- expect_lint/preferTimestamptz
create table events (
    a timestamp /* local */ without time zone
);
