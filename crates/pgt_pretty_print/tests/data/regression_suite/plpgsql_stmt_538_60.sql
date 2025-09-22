do $$
declare
  t test_01;
begin
  select 1 into t; -- should fail;
end;
$$;
