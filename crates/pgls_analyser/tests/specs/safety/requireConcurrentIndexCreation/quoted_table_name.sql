-- "Users" and "App" are new, but users and app.events are different tables.
CREATE TABLE "Users" (id bigint);
CREATE INDEX "Users_id_idx" ON "Users" (id);
-- expect_lint/requireConcurrentIndexCreation
CREATE INDEX users_id_idx ON users (id);
CREATE TABLE "App".events (id bigint);
-- expect_lint/requireConcurrentIndexCreation
CREATE INDEX events_id_idx ON app.events (id);
