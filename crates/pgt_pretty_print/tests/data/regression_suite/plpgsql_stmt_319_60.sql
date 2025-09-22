create function f1(x anycompatible, y anycompatible) returns anycompatiblearray as $$
begin
  return array[x, y];
end$$ language plpgsql;
