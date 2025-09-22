create function pg_temp.whoami() returns text
  as $$select 'temp'::text$$ language sql;
