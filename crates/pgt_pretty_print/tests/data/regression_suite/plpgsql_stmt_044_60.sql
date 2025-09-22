create trigger tg_hslot_biu before insert or update
    on HSlot for each row execute procedure tg_hslot_biu();
