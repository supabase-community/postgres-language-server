create function whoami() returns trigger language plpgsql
as $$
begin
  raise notice 'I am %', current_user;
  return null;
end;
$$;
