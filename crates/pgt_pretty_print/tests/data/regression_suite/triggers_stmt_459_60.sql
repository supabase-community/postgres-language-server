create table parted_trig_1 partition of parted_trig for values from (0) to (1000)
   partition by range (a);
