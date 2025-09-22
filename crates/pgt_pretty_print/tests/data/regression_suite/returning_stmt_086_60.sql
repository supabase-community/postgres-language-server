INSERT INTO foo VALUES (5, 'subquery test')
  RETURNING (SELECT max(old.f4 + x) FROM generate_series(1, 10) x) old_max,
            (SELECT max(new.f4 + x) FROM generate_series(1, 10) x) new_max;
