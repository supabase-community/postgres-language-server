create trigger tg_wslot_biu before insert or update
    on WSlot for each row execute procedure tg_wslot_biu();
