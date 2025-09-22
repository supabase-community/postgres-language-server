do $$
declare y int := x + 1;  -- error
        x int := 42;
begin
  raise notice 'x = %, y = %', x, y;
end;
$$;
