create or replace function f1(inout i int) as $$
begin
  i := i+1;
end$$ language plpgsql;
