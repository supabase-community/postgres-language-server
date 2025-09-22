create table idxpart1 partition of idxpart (i) for values with (modulus 2, remainder 1);
