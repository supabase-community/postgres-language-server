SELECT relname, d.* FROM d, pg_class where d.tableoid = pg_class.oid;
