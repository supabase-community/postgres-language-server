create function current_function(text)
returns regprocedure as $$
declare
  fn_oid regprocedure;
begin
  get diagnostics fn_oid = pg_routine_oid;
  return fn_oid;
end;
$$ language plpgsql;
