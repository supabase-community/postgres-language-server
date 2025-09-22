CREATE TEMP VIEW v_window AS
	SELECT i, min(i) over (order by i range between '1 day' preceding and '10 days' following) as min_i
  FROM generate_series(now(), now()+'100 days'::interval, '1 hour') i;
