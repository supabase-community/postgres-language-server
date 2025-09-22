create function f1(a anyelement, b anyarray,
                   c anycompatible, d anycompatible,
                   OUT x anyarray, OUT y anycompatiblearray)
as $$
begin
  x := a || b;
  y := array[c, d];
end$$ language plpgsql;
