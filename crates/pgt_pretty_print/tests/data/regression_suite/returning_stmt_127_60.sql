UPDATE foo_parted SET a = 4, b = b + 1, c = c || '->P4' WHERE a = 3
  RETURNING old.tableoid::regclass, old.ctid, old.*,
            new.tableoid::regclass, new.ctid, new.*, *;
