create function refcursor_test1(refcursor) returns refcursor as $$
begin
    perform return_refcursor($1);
    return $1;
end
$$ language plpgsql;
