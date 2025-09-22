UPDATE foo SET f4 = 100 WHERE f1 = 5
  RETURNING (SELECT old.f4 = new.f4),
            (SELECT max(old.f4 + x) FROM generate_series(1, 10) x) old_max,
            (SELECT max(new.f4 + x) FROM generate_series(1, 10) x) new_max;
