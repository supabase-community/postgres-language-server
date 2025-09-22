create function missing_return_expr() returns int as $$
begin
    perform 2+2;
end;$$ language plpgsql;
