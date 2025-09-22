create trigger tg_chkslotname before insert
    on WSlot for each row execute procedure tg_chkslotname('WS');
