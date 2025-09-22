CREATE TABLE part_bogus_expr_fail PARTITION OF range_parted
  FOR VALUES FROM (generate_series(1, 3)) TO ('2019-01-01');
