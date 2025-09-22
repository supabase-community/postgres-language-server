create function return_refcursor(rc refcursor) returns refcursor as $$
begin
    open rc for select a from rc_test;
    return rc;
end
$$ language plpgsql;
