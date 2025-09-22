create or replace function composrec() returns record as $$
begin
  return (1, 'hello');
end;
$$ language plpgsql;
