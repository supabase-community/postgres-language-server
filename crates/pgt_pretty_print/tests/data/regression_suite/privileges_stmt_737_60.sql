CREATE INDEX sro_pidx ON sro_ptab ((sro_ifun(a) + sro_ifun(0)))
	WHERE sro_ifun(a + 10) > sro_ifun(10);
