do $$
declare var text := 'some value';
begin
  assert 1=0, format('assertion failed, var = "%s"', var);
end;
$$;
