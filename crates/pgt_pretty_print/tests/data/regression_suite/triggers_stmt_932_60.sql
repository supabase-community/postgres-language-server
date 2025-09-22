create trigger but_trigger after update on convslot_test_child
referencing new table as new_table
for each statement execute function convslot_trig2();
