do $$
declare x int := 42;
begin
  declare y int := x + 1;
          x int := x + 2;
          z int := x * 10;
  begin
    raise notice 'x = %, y = %, z = %', x, y, z;
  end;
end;
$$;
