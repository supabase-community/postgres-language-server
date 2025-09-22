create function f1(x anyarray) returns anyarray as $$
begin
  return x;
end$$ language plpgsql;
