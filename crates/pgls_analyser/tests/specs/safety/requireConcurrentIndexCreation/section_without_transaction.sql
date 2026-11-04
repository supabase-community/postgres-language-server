-- The statements surely run outside a transaction, so the fixes are safe.
-- section: up transaction:false queries:separate
-- expect_lint/requireConcurrentIndexCreation
create index users_email_idx on users (email);
-- expect_lint/requireConcurrentIndexCreation
create index users_name_idx on users (name);
