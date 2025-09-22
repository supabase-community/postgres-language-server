create function funcB() returns trigger as $$
begin
  raise notice 'hello from funcB';
  return null;
end; $$ language plpgsql;
