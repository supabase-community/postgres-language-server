create function fail() returns int language plpgsql as $$
begin
  return 1/0;
end
$$;
