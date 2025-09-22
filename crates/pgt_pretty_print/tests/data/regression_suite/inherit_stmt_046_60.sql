SELECT relname, b.* FROM ONLY b, pg_class where b.tableoid = pg_class.oid;
