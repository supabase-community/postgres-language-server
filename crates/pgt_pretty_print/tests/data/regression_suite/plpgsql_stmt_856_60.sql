do $$
declare
  fn_oid oid;
begin
  get diagnostics fn_oid = pg_routine_oid;
  raise notice 'pg_routine_oid = %', fn_oid;
end;
$$;
