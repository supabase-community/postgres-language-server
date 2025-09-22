create or replace function rttest()
returns setof int as $$
begin
  return query execute 'select 10 into no_such_table';
end;
$$ language plpgsql;
