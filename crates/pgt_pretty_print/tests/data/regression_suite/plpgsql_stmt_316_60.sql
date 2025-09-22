create function f1(x anyrange) returns anyarray as $$
begin
  return array[lower(x), upper(x)];
end$$ language plpgsql;
