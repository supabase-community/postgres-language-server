create function f1(x anyelement) returns anyrange as $$
begin
  return array[x + 1, x + 2];
end$$ language plpgsql;
