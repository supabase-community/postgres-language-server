create function f1(in i int, out j int) as $$
begin
  j := i+1;
  return;
end$$ language plpgsql;
