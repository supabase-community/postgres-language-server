create or replace function array_elem_check(int) returns int as $$
declare
  x orderedpair := '{1,2}';
begin
  x[2] := $1;
  return x[2];
end$$ language plpgsql;
