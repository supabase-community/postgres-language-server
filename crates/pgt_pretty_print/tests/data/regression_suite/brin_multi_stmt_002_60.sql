INSERT INTO brintest_multi (inetcol, cidrcol) SELECT
	inet 'fe80::6e40:8ff:fea9:8c46' + tenthous,
	cidr 'fe80::6e40:8ff:fea9:8c46' + tenthous
FROM tenk1 ORDER BY thousand, tenthous LIMIT 25;
