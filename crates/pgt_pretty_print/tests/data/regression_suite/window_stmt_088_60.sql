CREATE OR REPLACE TEMP VIEW v_window AS
	SELECT i, sum(i) over (order by i groups between 1 preceding and 1 following) as sum_rows FROM generate_series(1, 10) i;
