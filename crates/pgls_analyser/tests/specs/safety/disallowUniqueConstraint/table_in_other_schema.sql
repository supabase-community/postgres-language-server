-- app.users is new, but public.users is not.
-- expect_lint/disallowUniqueConstraint
CREATE TABLE app.users (id bigint, email text);
ALTER TABLE public.users ADD CONSTRAINT users_email_key UNIQUE (email);
