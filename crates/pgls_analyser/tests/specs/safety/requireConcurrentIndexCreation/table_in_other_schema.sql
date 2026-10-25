-- app.users is new, but public.users is not.
-- expect_lint/requireConcurrentIndexCreation
CREATE TABLE app.users (id bigint, email text);
CREATE INDEX users_email_idx ON public.users (email);
