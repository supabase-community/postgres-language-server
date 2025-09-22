create trigger parted_conflict_update
    after update on parted_conflict
    referencing new table as inserted
    for each statement
    execute procedure parted_conflict_update_func();
