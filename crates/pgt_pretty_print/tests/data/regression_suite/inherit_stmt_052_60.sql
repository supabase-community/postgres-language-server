SELECT relname, c.* FROM c, pg_class where c.tableoid = pg_class.oid;
