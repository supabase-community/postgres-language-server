create function funcA() returns trigger as $$
begin
  raise notice 'hello from funcA';
  return null;
end; $$ language plpgsql;
