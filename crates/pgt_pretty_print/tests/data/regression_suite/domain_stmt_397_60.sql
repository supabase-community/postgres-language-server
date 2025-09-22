create or replace function array_elem_check(numeric) returns numeric as $$
declare
  x numeric(4,2)[1];
begin
  x[1] := $1;
  return x[1];
end$$ language plpgsql;
