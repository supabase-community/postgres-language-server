create or replace function returnqueryf()
returns setof tabwithcols as $$
begin
  return query select * from tabwithcols;
  return query execute 'select * from tabwithcols';
end;
$$ language plpgsql;
