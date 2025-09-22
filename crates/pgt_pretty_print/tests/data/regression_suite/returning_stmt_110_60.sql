DELETE FROM zerocol
  RETURNING old.tableoid::regclass, old.ctid,
            new.tableoid::regclass, new.ctid, ctid, *;
