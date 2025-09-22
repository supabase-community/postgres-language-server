UPDATE joinview SET f3 = f3 + 1 WHERE f3 = 57
  RETURNING old.*, new.*, *, new.f3 - old.f3 AS delta_f3;
