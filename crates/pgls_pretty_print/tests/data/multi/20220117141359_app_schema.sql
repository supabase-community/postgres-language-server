create schema app;

grant usage on schema app to authenticated, anon;

alter default privileges in schema app grant select on tables to authenticated, anon;
