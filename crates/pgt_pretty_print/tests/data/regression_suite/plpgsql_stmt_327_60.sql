create function f1(x anycompatiblerange, y anycompatiblearray) returns anycompatiblerange as $$
begin
  return x;
end$$ language plpgsql;
