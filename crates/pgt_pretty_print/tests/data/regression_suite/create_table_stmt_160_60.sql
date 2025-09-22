CREATE TABLE part_bogus_expr_fail PARTITION OF range_parted
  FOR VALUES FROM ((select 1)) TO ('2019-01-01');
