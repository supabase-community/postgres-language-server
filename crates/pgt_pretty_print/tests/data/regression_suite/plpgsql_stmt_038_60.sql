create trigger tg_hub_a after insert or update or delete
    on Hub for each row execute procedure tg_hub_a();
