-- An unqualified name is the first table with that name on the search path, also outside `app`.
create schema app;
set search_path = app, public;
create table events (id bigint, name text);
create index events_name_idx on events (name);
create table public.users (id bigint, email text);
create index users_email_idx on users (email);
-- `public.users` is new, but `app.users` isn't.
-- expect_lint/requireConcurrentIndexCreation
create index app_users_email_idx on app.users (email);
