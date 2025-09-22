create trigger tg_chkbacklink before insert or update
    on WSlot for each row execute procedure tg_chkbacklink();
