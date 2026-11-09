-- "Users" and "App" are new, but users and app.events are different tables.
CREATE TABLE "Users" (id bigint);
ALTER TABLE "Users" ADD CONSTRAINT "Users_id_key" UNIQUE (id);
-- expect_lint/disallowUniqueConstraint
ALTER TABLE users ADD CONSTRAINT users_id_key UNIQUE (id);
CREATE TABLE "App".events (id bigint);
-- expect_lint/disallowUniqueConstraint
ALTER TABLE app.events ADD CONSTRAINT events_id_key UNIQUE (id);
