create temp table idxpart1_temp partition of idxpart_temp
  for values from (0) to (10);
