create trigger tg_chkbacklink before insert or update
    on PLine for each row execute procedure tg_chkbacklink();
