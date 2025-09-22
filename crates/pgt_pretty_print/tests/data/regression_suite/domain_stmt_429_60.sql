create or replace function dom_check(int) returns di as $$
declare d di;
begin
  d := $1;
  return d;
end
$$ language plpgsql immutable;
