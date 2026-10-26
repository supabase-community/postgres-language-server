-- expect_lint/requireConcurrentIndexCreation
create index users_email_idx on users (email);
-- Postgres 18 syntax that the Postgres 17 parser can't parse. The file still has two
-- statements and is assumed to run in a transaction, so there is no fix.
alter table users add constraint users_age_check check (age > 0) not enforced;
