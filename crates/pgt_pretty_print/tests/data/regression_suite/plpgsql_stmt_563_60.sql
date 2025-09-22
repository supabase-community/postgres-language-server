create or replace function ret_query2(lim int) returns setof record_type as $$
begin
    return query select fipshash(s.x::text), s.x, s.x > 0
                 from generate_series(-8, lim) s (x) where s.x % 2 = 0;
end;
$$ language plpgsql;
