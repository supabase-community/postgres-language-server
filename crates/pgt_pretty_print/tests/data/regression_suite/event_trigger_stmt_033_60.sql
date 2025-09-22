create function f1() returns int
language plpgsql
as $$
begin
  create table event_trigger_fire6 (a int);
  return 0;
end $$;
