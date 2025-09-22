create trigger tg_room_au after update
    on Room for each row execute procedure tg_room_au();
