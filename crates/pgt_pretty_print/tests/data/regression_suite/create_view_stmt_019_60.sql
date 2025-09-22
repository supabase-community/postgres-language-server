CREATE OR REPLACE VIEW viewtest AS
	SELECT a, b, c, d FROM viewtest_tbl WHERE a > 5 ORDER BY b DESC;
