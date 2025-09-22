CREATE OR REPLACE VIEW viewtest AS
	SELECT a, b, c, d COLLATE "POSIX" FROM viewtest_tbl;
