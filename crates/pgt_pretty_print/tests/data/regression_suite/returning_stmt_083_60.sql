UPDATE foo SET f4 = 100 WHERE f1 = 5
  RETURNING old.tableoid::regclass, old.ctid, old.*, old,
            new.tableoid::regclass, new.ctid, new.*, new,
            old.f4::text||'->'||new.f4::text AS change;
