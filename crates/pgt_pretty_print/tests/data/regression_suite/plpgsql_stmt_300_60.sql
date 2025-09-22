create function f1(x anyelement) returns anyelement as $$
begin
  return x + 1;
end$$ language plpgsql;
