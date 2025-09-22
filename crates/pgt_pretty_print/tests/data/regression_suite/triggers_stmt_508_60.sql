create function parted_trigfunc() returns trigger language plpgsql as $$
begin
  new.a = new.a + 1;
  return new;
end;
$$;
