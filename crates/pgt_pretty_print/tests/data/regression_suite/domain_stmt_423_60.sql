create function dom_check(int) returns di as $$
declare d di;
begin
  d := $1::di;
  return d;
end
$$ language plpgsql immutable;
