create function return_unnamed_refcursor() returns refcursor as $$
declare
    rc refcursor;
begin
    open rc for select a from rc_test;
    return rc;
end
$$ language plpgsql;
