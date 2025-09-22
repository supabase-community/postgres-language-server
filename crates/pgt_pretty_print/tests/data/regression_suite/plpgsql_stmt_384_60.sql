create function use_refcursor(rc refcursor) returns int as $$
declare
    rc refcursor;
    x record;
begin
    rc := return_unnamed_refcursor();
    fetch next from rc into x;
    return x.a;
end
$$ language plpgsql;
