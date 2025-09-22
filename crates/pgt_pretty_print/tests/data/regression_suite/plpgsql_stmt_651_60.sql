create or replace function raise_test() returns void as $$
begin
  raise;
end;
$$ language plpgsql;
