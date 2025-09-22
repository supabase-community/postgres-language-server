create function namedparmcursor_test9(p1 int) returns int4 as $$
declare
  c1 cursor (p1 int, p2 int, debug int) for
    select count(*) from tenk1 where thousand = p1 and tenthous = p2
      and four = debug;
  p2 int4 := 1006;
  n int4;
begin
  -- use both supported syntaxes for named arguments
  open c1 (p1 := p1, p2 => p2, debug => 2);
  fetch c1 into n;
  return n;
end $$ language plpgsql;
