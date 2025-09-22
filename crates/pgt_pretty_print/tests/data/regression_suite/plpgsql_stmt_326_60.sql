create function f1(x anycompatible) returns anycompatiblerange as $$
begin
  return array[x + 1, x + 2];
end$$ language plpgsql;
