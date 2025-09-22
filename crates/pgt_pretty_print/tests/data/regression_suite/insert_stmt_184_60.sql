create table hpart3 partition of hash_parted for values with (modulus 4, remainder 3);
