create or replace function doubledecrement(p1 pos_int) returns pos_int as $$
declare v pos_int := 1;
begin
    v := p1 - 1;
    return v - 1;
end$$ language plpgsql;
