create table idxpart2 partition of idxpart for values from (10) to (100)
	partition by range (b);
