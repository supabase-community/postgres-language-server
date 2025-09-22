CREATE POLICY priv_test_tbl_pol ON tststats.priv_test_tbl USING (2 * a < 0);
