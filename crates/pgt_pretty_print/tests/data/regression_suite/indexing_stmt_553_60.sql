select indexrelid::regclass, indisvalid, indisreplident,
       indrelid::regclass, inhparent::regclass
  from pg_index idx left join
       pg_inherits inh on (idx.indexrelid = inh.inhrelid)
  where indexrelid::regclass::text like 'parted_replica%'
  order by indexrelid::regclass::text collate "C";
