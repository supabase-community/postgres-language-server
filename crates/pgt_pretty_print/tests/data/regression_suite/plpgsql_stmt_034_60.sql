create trigger tg_system_au after update
    on System for each row execute procedure tg_system_au();
