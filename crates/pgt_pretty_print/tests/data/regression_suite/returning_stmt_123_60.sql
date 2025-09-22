INSERT INTO foo_parted
  VALUES (1, 17.1, 'P1'), (2, 17.2, 'P2'), (3, 17.3, 'P3'), (4, 17.4, 'P4')
  RETURNING old.tableoid::regclass, old.ctid, old.*,
            new.tableoid::regclass, new.ctid, new.*, *;
