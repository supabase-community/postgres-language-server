create function void_return_expr() returns void as $$
begin
    return 5;
end;$$ language plpgsql;
