CREATE RULE foo_del_rule AS ON DELETE TO foo DO INSTEAD
  UPDATE foo SET f2 = f2||' (deleted)', f3 = -1, f4 = -1 WHERE f1 = OLD.f1
  RETURNING *;
