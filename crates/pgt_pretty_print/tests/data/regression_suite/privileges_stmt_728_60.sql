CREATE INDEX sro_brin ON sro_tab USING brin ((sro_ifun(a) + sro_ifun(0)));
