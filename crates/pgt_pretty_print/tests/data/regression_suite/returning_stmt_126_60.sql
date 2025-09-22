UPDATE foo_parted SET a = 3, b = b + 1, c = c || '->P3' WHERE a = 1
  RETURNING old.tableoid::regclass, old.ctid, old.*,
            new.tableoid::regclass, new.ctid, new.*, *;
