create table parted_trigger_3 (b text, a int) partition by range (length(b));
