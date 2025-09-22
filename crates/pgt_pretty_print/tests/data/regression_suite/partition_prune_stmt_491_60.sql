select * from stable_qual_pruning
  where a = any(null::timestamptz[]);
