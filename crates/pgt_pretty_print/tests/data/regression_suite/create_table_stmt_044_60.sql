SELECT relpersistence,
  pg_filenode_relation (reltablespace, pg_relation_filenode(oid))
  FROM pg_class
  WHERE relname = 'relation_filenode_check';
