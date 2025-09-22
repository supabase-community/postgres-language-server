CREATE POLICY priv_test_parent_tbl_pol ON tststats.priv_test_parent_tbl USING (2 * a < 0);
