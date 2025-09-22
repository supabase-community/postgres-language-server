create function f1(in i int, out j int, out k text) returns setof record as $$
begin
  j := i+1;
  k := 'foo';
  return next;
  j := j+1;
  k := 'foot';
  return next;
end$$ language plpgsql;
