select * from stable_qual_pruning
  where a = any(array['2000-02-01', localtimestamp]::timestamp[]);
