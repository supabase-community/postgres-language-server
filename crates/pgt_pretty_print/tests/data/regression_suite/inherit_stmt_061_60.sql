SELECT relname, b.* FROM b, pg_class where b.tableoid = pg_class.oid;
