-- expect_lint/requireConcurrentIndexCreation
create unique index if not exists users_email_idx on users (lower(email));
