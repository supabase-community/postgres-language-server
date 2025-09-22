CREATE RULE rw_view16_upd_rule AS ON UPDATE TO rw_view16
  WHERE OLD.a > 0 DO INSTEAD UPDATE base_tbl SET b=NEW.b WHERE a=OLD.a;
