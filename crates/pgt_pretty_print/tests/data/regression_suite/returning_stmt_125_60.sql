UPDATE foo_parted SET a = 1, b = b + 1, c = c || '->P1' WHERE a = 3
  RETURNING old.tableoid::regclass, old.ctid, old.*,
            new.tableoid::regclass, new.ctid, new.*, *;
