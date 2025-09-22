CREATE TABLE part_bogus_expr_fail PARTITION OF range_parted
  FOR VALUES FROM (max('2019-02-01'::date)) TO ('2019-01-01');
