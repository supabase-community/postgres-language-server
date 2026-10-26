-- DROP INDEX CONCURRENTLY supports neither multiple indexes nor CASCADE
-- expect_lint/requireConcurrentIndexDeletion
drop index users_email_idx, users_name_idx;
-- expect_lint/requireConcurrentIndexDeletion
drop index if exists users_email_idx cascade;
