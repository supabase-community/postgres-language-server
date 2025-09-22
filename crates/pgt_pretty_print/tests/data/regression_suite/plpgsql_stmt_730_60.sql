create function cast_invoker(integer) returns date as $$
begin
  return $1;
end$$ language plpgsql;
