create function f1(x anycompatiblerange, y anycompatible, z anycompatible) returns anycompatiblearray as $$
begin
  return array[lower(x), upper(x), y, z];
end$$ language plpgsql;
