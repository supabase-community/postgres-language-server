create function part_ins_func() returns trigger language plpgsql as $$
begin
  return new;
end;
$$;
