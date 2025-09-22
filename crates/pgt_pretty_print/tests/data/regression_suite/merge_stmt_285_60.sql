CREATE TABLE pa_target (tid integer, balance float, val text)
	PARTITION BY LIST (tid);
