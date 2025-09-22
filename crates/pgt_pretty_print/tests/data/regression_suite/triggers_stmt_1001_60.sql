create or replace function whoami() returns trigger language plpgsql
as $$
begin
  raise notice 'I am %', current_user;
  perform 1 / 0;
  return null;
end;
$$;
