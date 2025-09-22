create or replace function parted_trigfunc() returns trigger language plpgsql as $$
begin
  new.a = new.a + new.b;
  return new;
end;
$$;
