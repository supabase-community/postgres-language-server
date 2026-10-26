-- expect_lint/preferTimestamptz
-- expect_lint/preferTimestamptz
-- expect_lint/preferTimestamptz
-- expect_lint/preferTimestamptz
create table events (
    id bigint primary key,
    created_at timestamp not null,
    updated_at timestamp(3) without time zone,
    deleted_at pg_catalog.timestamp,
    seen_at timestamp[],
    archived_at timestamptz
);
