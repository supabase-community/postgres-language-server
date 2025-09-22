create trigger convslot_test_parent_update
    after update on convslot_test_parent
    referencing old table as old_rows new table as new_rows
    for each statement execute procedure convslot_trig4();
