SELECT relname, c.* FROM ONLY c, pg_class where c.tableoid = pg_class.oid;
