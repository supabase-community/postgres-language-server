-- CONCURRENTLY can't run in a transaction block, so there is no fix
begin;
-- expect_lint/requireConcurrentIndexCreation
create index users_email_idx on users (email);
commit;
