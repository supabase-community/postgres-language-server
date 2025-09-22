UPDATE joinview SET f3 = f3 + 1, f4 = 7 WHERE f3 = 58
  RETURNING old.*, new.*, *, new.f3 - old.f3 AS delta_f3;
