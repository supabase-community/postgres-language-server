create function missing_return_expr() returns int as $$
begin
    return ;
end;$$ language plpgsql;
