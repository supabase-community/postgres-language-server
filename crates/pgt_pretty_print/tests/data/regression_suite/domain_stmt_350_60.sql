create or replace function doubledecrement(p1 pos_int) returns pos_int as $$
declare v pos_int := 0;
begin
    return p1;
end$$ language plpgsql;
