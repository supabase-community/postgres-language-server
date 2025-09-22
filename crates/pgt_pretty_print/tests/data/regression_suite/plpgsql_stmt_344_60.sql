create function f1(in i int, out j int) returns setof int as $$
begin
  j := i+1;
  return next;
  j := i+2;
  return next;
  return;
end$$ language plpgsql;
