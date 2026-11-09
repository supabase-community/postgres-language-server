-- A table created in the transaction doesn't need a timeout, also outside `public`.
create schema app;
set search_path = app, public;
create table events (id bigint);
alter table events add column name text;
create index events_name_idx on events (name);
create table public.users (id bigint);
alter table users add column email text;
-- `public.users` is new, but `app.users` isn't.
-- expect_lint/lockTimeoutWarning
alter table app.users add column email text;
