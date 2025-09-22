create function rttest()
returns setof int as $$
declare rc int;
begin
  return query values(10),(20);
  get diagnostics rc = row_count;
  raise notice '% %', found, rc;
  return query select * from (values(10),(20)) f(a) where false;
  get diagnostics rc = row_count;
  raise notice '% %', found, rc;
  return query execute 'values(10),(20)';
  get diagnostics rc = row_count;
  raise notice '% %', found, rc;
  return query execute 'select * from (values(10),(20)) f(a) where false';
  get diagnostics rc = row_count;
  raise notice '% %', found, rc;
end;
$$ language plpgsql;
