SELECT shobj_description(d.oid, 'pg_database') as description_before
  FROM pg_database d WHERE datname = current_database() ;
