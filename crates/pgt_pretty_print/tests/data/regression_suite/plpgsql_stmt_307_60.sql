create function f1(x anyarray) returns anyelement as $$
begin
  return x[1];
end$$ language plpgsql;
