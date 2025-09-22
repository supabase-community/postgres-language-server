create table bool_rp_true_1k partition of bool_rp for values from (true,0) to (true,1000);
