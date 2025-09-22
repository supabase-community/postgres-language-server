create trigger bdt_trigger after delete on convslot_test_child
referencing old table as old_table
for each statement execute function convslot_trig1();
