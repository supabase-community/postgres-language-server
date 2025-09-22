CREATE RULE base_tbl_ins_rule AS ON INSERT TO base_tbl
  WHERE EXISTS (SELECT 1 FROM base_tbl t WHERE t.id = new.id)
  DO INSTEAD
    UPDATE base_tbl SET data = new.data, deleted = false WHERE id = new.id;
