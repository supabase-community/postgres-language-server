SELECT reltuples, relhassubclass
  FROM pg_class WHERE oid = 'past_inh_parent'::regclass;
