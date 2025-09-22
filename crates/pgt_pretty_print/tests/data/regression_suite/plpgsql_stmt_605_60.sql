create or replace function compos() returns compostype as $$
begin
  return (1, 'hello');
end;
$$ language plpgsql;
