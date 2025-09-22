create function error_trap_test() returns text as $$
begin
  perform zero_divide();
  return 'no error detected!';
exception when division_by_zero then
  return 'division_by_zero detected';
end;
$$ language plpgsql parallel safe;
