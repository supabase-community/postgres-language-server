do $$
declare x int := 42;
        y int := x + 1;
begin
  raise notice 'x = %, y = %', x, y;
end;
$$;
