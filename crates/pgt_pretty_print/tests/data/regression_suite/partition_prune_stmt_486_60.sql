select * from stable_qual_pruning
  where a = any(array['2010-02-01', '2020-01-01']::timestamp[]);
