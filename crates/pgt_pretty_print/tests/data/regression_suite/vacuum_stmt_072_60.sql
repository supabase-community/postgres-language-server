SELECT reltuples, relhassubclass
  FROM pg_class WHERE oid = 'past_parted'::regclass;
