create table volatile_partbound_test2 partition of volatile_partbound_test for values from (current_timestamp) to (maxvalue);
