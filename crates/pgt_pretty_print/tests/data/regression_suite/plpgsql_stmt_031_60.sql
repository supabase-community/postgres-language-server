create function tg_pslot_biu() returns trigger as $proc$
declare
    pfrec	record;
    ps          alias for new;
begin
    select into pfrec * from PField where name = ps.pfname;
    if not found then
        raise exception $$Patchfield "%" does not exist$$, ps.pfname;
    end if;
    return ps;
end;
$proc$ language plpgsql;
