CREATE TABLE FKTABLE (
  tid int, id int,
  fk_id_del_set_null int,
  fk_id_del_set_default int DEFAULT 0,
  FOREIGN KEY (tid, fk_id_del_set_null) REFERENCES PKTABLE ON DELETE SET NULL (fk_id_del_set_null),
  -- this tests handling of duplicate entries in SET DEFAULT column list
  FOREIGN KEY (tid, fk_id_del_set_default) REFERENCES PKTABLE ON DELETE SET DEFAULT (fk_id_del_set_default, fk_id_del_set_default)
);
