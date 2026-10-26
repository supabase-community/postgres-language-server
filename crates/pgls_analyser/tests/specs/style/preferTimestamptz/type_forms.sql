-- expect_lint/preferTimestamptz
-- expect_lint/preferTimestamptz
-- expect_lint/preferTimestamptz
-- expect_lint/preferTimestamptz
CREATE TABLE events (
    a TIMESTAMP WITHOUT TIME ZONE,
    b timestamp without time zone[],
    c "timestamp",
    d timestamp /* precision */ (3) without time zone,
    e myschema.timestamp
);
