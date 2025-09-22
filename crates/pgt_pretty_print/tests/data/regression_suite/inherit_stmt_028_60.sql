SELECT relname, a.* FROM a, pg_class where a.tableoid = pg_class.oid;
