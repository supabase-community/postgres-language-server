create table volatile_partbound_test1 partition of volatile_partbound_test for values from (minvalue) to (current_timestamp);
