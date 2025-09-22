create function tg_chkbacklink() returns trigger as '
begin
    if new.backlink isnull then
        new.backlink := '''';
    end if;
    return new;
end;
' language plpgsql;
