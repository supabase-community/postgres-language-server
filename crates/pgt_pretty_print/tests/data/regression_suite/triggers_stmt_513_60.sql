create or replace function parted_trigfunc() returns trigger language plpgsql as $$
begin
  new.b = new.b + 1;
  return new;
end;
$$;
