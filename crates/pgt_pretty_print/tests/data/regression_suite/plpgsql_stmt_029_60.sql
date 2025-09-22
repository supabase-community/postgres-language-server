create function tg_pfield_ad() returns trigger as '
begin
    delete from PSlot where pfname = old.name;
    return old;
end;
' language plpgsql;
