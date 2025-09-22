create or replace function compos() returns compostype as $$
begin
  return 1 + 1;
end;
$$ language plpgsql;
