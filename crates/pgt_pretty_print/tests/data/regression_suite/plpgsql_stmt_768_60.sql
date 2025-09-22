do $$
declare x int := x + 1;  -- error
begin
  raise notice 'x = %', x;
end;
$$;
