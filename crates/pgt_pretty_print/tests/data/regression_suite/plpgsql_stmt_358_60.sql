create function duplic(in i anycompatiblerange, out j anycompatible, out k anycompatiblearray) as $$
begin
  j := lower(i);
  k := array[lower(i),upper(i)];
  return;
end$$ language plpgsql;
