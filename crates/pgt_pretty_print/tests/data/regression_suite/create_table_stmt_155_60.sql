CREATE TABLE part_bogus_expr_fail PARTITION OF range_parted
  FOR VALUES FROM (somename.somename) TO ('2019-01-01');
