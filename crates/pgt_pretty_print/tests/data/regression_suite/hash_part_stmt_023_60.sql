create table text_hashp1 partition of text_hashp for values with (modulus 2, remainder 1);
