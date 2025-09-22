CREATE OR REPLACE VIEW viewtest AS
	SELECT a, b::numeric, c, d FROM viewtest_tbl;
