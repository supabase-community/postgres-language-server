create function f1(in i int, out j int, out k text) as $$
begin
  j := i;
  j := j+1;
  k := 'foo';
end$$ language plpgsql;
