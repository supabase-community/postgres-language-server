create trigger tg_hslot_bu before update
    on HSlot for each row execute procedure tg_hslot_bu();
