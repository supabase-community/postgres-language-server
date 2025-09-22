SELECT unnest(pg_get_acl('pg_class'::regclass, 'atest5'::regclass::oid, 3));
