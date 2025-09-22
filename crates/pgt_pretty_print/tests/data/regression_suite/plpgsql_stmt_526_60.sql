do $$
declare x int;
begin
  select v from generate_series(1,2) g(v) into x;
end;
$$;
