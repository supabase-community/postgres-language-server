create table bool_rp_false_1k partition of bool_rp for values from (false,0) to (false,1000);
