SELECT reltuples, relhassubclass
  FROM pg_class WHERE oid = 'past_inh_db_parent'::regclass;
