-- expect_lint/requireConcurrentIndexCreation
create index users_email_idx on users (email);
-- A statement the parser can't parse (the CHECK expression lacks its parentheses). The file
-- still has two statements and is assumed to run in a transaction, so there is no fix.
alter table users add constraint users_age_check check age > 0;
