create trigger tg_pslot_biu before insert or update
    on PSlot for each row execute procedure tg_pslot_biu();
