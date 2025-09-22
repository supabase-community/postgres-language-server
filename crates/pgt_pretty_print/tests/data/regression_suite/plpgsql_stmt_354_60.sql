create function duplic(in i anyelement, out j anyelement, out k anyarray) as $$
begin
  j := i;
  k := array[j,j];
  return;
end$$ language plpgsql;
