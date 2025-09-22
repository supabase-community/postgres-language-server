create function void_return_expr() returns void as $$
begin
    perform 2+2;
end;$$ language plpgsql;
