-- expect_lint/preferTimestamptz
alter table events alter column created_at type TIMESTAMP WITHOUT TIME ZONE using created_at;
