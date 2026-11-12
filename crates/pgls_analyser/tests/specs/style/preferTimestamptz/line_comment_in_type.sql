-- The fix keeps a line comment in the type on a line of its own, so it doesn't comment out the
-- constraints after the type.
-- expect_lint/preferTimestamptz
create table events (
    a timestamp -- local
    without time zone not null
);
