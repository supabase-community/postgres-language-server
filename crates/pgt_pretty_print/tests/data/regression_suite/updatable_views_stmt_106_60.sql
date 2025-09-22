CREATE RULE base_tbl_log AS ON INSERT TO rw_view1 DO ALSO
  INSERT INTO base_tbl_hist(a,b) VALUES(new.a, new.b);
