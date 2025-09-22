SELECT * FROM tststats.priv_test_tbl
  WHERE a = 1 and tststats.priv_test_tbl.* > (1, 1) is not null;
