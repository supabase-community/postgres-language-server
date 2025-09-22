do $$
declare a int[] := array[1,2];
begin
  a := a || 3;
  raise notice 'a = %', a;
end$$;
