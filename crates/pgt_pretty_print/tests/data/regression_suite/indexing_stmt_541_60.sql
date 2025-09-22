select indexrelid::regclass, indisvalid,
       indrelid::regclass, inhparent::regclass
  from pg_index idx left join
       pg_inherits inh on (idx.indexrelid = inh.inhrelid)
  where indexrelid::regclass::text like 'parted_isvalid%'
  order by indexrelid::regclass::text collate "C";
