CREATE TABLE part_bogus_expr_fail PARTITION OF range_parted
  FOR VALUES FROM (max(somename)) TO ('2019-01-01');
