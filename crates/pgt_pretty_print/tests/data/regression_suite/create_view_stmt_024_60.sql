CREATE OR REPLACE VIEW viewtest AS
	SELECT a, b, c::numeric(10,2), d FROM viewtest_tbl;
