create trigger but_trigger2 after update on convslot_test_child
referencing old table as old_table new table as new_table
for each statement execute function convslot_trig3();
