create function bad_sql1() returns int as $$
declare a int;
begin
    a := 5;
    Johnny Yuma;
    a := 10;
    return a;
end$$ language plpgsql;
